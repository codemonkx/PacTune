use gtk::{gdk::Texture, gio, glib, prelude::*};
use lofty::tag::{ItemValue, Tag};
use lru::LruCache;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use std::{
    collections::HashMap,
    fs,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

// A huge lines of code (with refactoring) borrowed from:
// https://gitlab.gnome.org/World/amberol/-/blob/main/src/audio/cover_cache.rs
// https://gitlab.gnome.org/World/amberol/-/blob/main/src/utils.rs

use crate::APP_ID;

static USER_CACHE_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| glib::user_cache_dir().join(APP_ID).join("covers"));

#[derive(Debug)]
pub struct CoverCache {
    disk_index: HashMap<String, PathBuf>,
    texture_cache: LruCache<String, Texture>,
}

impl CoverCache {
    const MAX_TEXTURES: usize = 30;

    pub fn global() -> &'static Mutex<CoverCache> {
        static CACHE: OnceLock<Mutex<CoverCache>> = OnceLock::new();

        CACHE.get_or_init(|| {
            let c = CoverCache::new();
            Mutex::new(c)
        })
    }

    fn new() -> Self {
        CoverCache {
            disk_index: HashMap::new(),
            texture_cache: LruCache::new(NonZeroUsize::new(Self::MAX_TEXTURES).unwrap()),
        }
    }

    pub fn get_texture(&mut self, uuid: &str) -> Option<Texture> {
        if let Some(texture) = self.texture_cache.get(uuid) {
            return Some(texture.clone());
        }

        let path = self.disk_index.get(uuid)?;
        let file = gio::File::for_path(path);

        let texture = Texture::from_file(&file).ok();

        if let Some(texture) = texture {
            self.texture_cache.put(uuid.to_string(), texture.clone());
            return Some(texture);
        }
        None
    }

    // Cover for MPRIS
    pub fn get_cache_path(&self, uuid: &str) -> Option<&PathBuf> {
        self.disk_index.get(uuid)
    }

    pub fn cover_art(&mut self, path: &Path, tag: &Tag) -> Option<String> {
        let mut album = None;
        let mut album_artist = None;
        let mut track_artist = None;

        fn get_text(value: &ItemValue) -> Option<String> {
            match value {
                ItemValue::Text(s) => Some(s.to_string()),
                _ => None,
            }
        }

        for item in tag.items() {
            match item.key() {
                lofty::prelude::ItemKey::AlbumTitle => album = get_text(item.value()),
                lofty::prelude::ItemKey::AlbumArtist => album_artist = get_text(item.value()),
                lofty::prelude::ItemKey::TrackArtist => track_artist = get_text(item.value()),
                _ => (),
            }
        }

        let mut hasher = Sha256::new();

        if let Some(album) = album {
            hasher.update(&album);
            if let Some(artist) = album_artist.or(track_artist) {
                hasher.update(&artist);
            }
            if let Some(parent) = path.parent() {
                let s = parent.to_str().unwrap();
                hasher.update(s);
            }
        } else {
            let s = path.to_str().unwrap();
            hasher.update(s);
        }

        let uuid = format!("{:x}", hasher.finalize());

        if self.disk_index.contains_key(&uuid) {
            return Some(uuid);
        }

        let cover_bytes = self.load_cover_art(tag, path.parent())?;
        let cache_path = save_cover_to_folder(&uuid, &cover_bytes)?;

        self.disk_index.insert(uuid.to_string(), cache_path);

        Some(uuid)
    }

    fn load_cover_art(&self, tag: &Tag, path: Option<&Path>) -> Option<glib::Bytes> {
        if let Some(picture) = tag.get_picture_type(lofty::picture::PictureType::CoverFront) {
            return Some(glib::Bytes::from(picture.data()));
        } else {
            for picture in tag.pictures() {
                let cover_art = match picture.pic_type() {
                    lofty::picture::PictureType::Other => Some(glib::Bytes::from(picture.data())),
                    _ => None,
                };

                if cover_art.is_some() {
                    return cover_art;
                }
            }
        }

        if let Some(path) = path {
            let ext_cover_basename = ["Cover", "cover", "Folder", "folder"];
            let ext_cover_ext = ["jpg", "png"];

            let ext_covers = ext_cover_basename
                .iter()
                .map(|&b| {
                    ext_cover_ext
                        .iter()
                        .map(move |&e| format!("{}.{}", &b, &e))
                        .collect::<Vec<_>>()
                })
                .fold(vec![], |mut v, mut dat| {
                    v.append(&mut dat);
                    v
                });
            for name in ext_covers {
                let mut cover_file = PathBuf::from(path);
                cover_file.push(name);

                let file = gio::File::for_path(&cover_file);
                if let Ok(res) = file.load_bytes(None::<&gio::Cancellable>) {
                    return Some(res.0);
                }
            }
        }

        None
    }

    pub fn clear(&mut self) {
        self.disk_index.clear();
        self.texture_cache.clear();
    }
}

fn save_cover_to_folder(uuid: &str, cover_bytes: &glib::Bytes) -> Option<PathBuf> {
    let cache_dir = &*USER_CACHE_DIR;

    fs::create_dir_all(cache_dir).ok();

    let ext = infer::get(cover_bytes.as_ref())
        .map(|t| t.extension())
        .unwrap_or("png");

    let cache_path = cache_dir.join(format!("{uuid}.{ext}"));

    if cache_path.exists() {
        return Some(cache_path);
    }

    fs::write(&cache_path, cover_bytes.as_ref()).ok();

    Some(cache_path)
}
