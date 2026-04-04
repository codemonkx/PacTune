use crate::backend::cover_cache::CoverCache;
use lofty::{
    prelude::*,
    tag::{Tag, TagType},
};
use lyrx::Lyrics;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct TrackInfo {
    path: PathBuf,
    artist: Option<String>,
    album: String,
    title: String,
    lyrics: Vec<(Option<f64>, String)>,
    disc_number: Option<u32>,
    track_number: Option<u32>,
    cover_uuid: Option<String>,
    duration: u64,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
}
impl TrackInfo {
    pub fn artist(&self) -> &str {
        if let Some(artist) = &self.artist {
            artist.as_ref()
        } else {
            "Unknown Artist"
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    pub fn album(&self) -> &str {
        self.album.as_ref()
    }

    pub fn lyrics(&self) -> &Vec<(Option<f64>, String)> {
        &self.lyrics
    }

    pub fn duration(&self) -> u64 {
        self.duration
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn disc_number(&self) -> Option<u32> {
        self.disc_number
    }

    pub fn track_number(&self) -> Option<u32> {
        self.track_number
    }

    pub fn cover_uuid(&self) -> Option<&str> {
        self.cover_uuid.as_deref()
    }

    pub fn new(path: &PathBuf) -> Self {
        let tagged_file = match lofty::read_from_path(path) {
            Ok(f) => f,
            Err(_) => {
                return TrackInfo {
                    path: path.to_path_buf(),
                    artist: None,
                    title: path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    // Album always has a value
                    album: path
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    lyrics: vec![],
                    duration: 0,
                    disc_number: None,
                    track_number: None,
                    cover_uuid: None,
                    bitrate: None,
                    sample_rate: None,
                    channels: None,
                };
            }
        };

        let mut cover_cache = CoverCache::global().lock().unwrap();

        let mut artist = None;
        let mut title = None;
        let mut album = None;
        let mut raw_lyrics = None;
        let mut track_number = None;
        let mut disc_number = None;
        let mut cover_uuid = None;

        if let Some(tag) = tagged_file.primary_tag() {
            artist = tag.artist().map(|s| s.to_string());
            title = tag.title().map(|s| s.to_string());
            album = tag.album().map(|s| s.to_string());
            raw_lyrics = Self::get_raw_lyrics(path, tag);
            track_number = tag.track();
            disc_number = tag.disk();
            cover_uuid = cover_cache.cover_art(path, tag);
        } else {
            for tag in tagged_file.tags() {
                artist = tag.artist().map(|s| s.to_string());
                title = tag.title().map(|s| s.to_string());
                album = tag.album().map(|s| s.to_string());
                raw_lyrics = Self::get_raw_lyrics(path, tag);
                track_number = tag.track();
                disc_number = tag.disk();
                cover_uuid = cover_cache.cover_art(path, tag);
            }
        }

        let return_title = title.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });

        let return_album = album.unwrap_or_else(|| {
            path.parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        });

        let lyrics: Vec<(Option<f64>, String)>;
        if let Some(raw_lyrics) = raw_lyrics {
            lyrics = Self::get_lyrics(raw_lyrics)
        } else {
            lyrics = vec![]
        }

        let properties = lofty::prelude::AudioFile::properties(&tagged_file);
        let duration = properties.duration().as_secs();
        let bitrate = properties.overall_bitrate();
        let sample_rate = properties.sample_rate();
        let channels = properties.channels();

        TrackInfo {
            path: path.to_path_buf(),
            title: return_title,
            artist,
            album: return_album,
            lyrics,
            duration,
            disc_number,
            track_number,
            cover_uuid,
            bitrate,
            sample_rate,
            channels,
        }
    }

    fn get_raw_lyrics(path: &Path, tag: &Tag) -> Option<String> {
        let mut lyrics: Option<String>;
        lyrics = lyrics_in_folder(path);

        fn lyrics_in_folder(path: &Path) -> Option<String> {
            let lrc_path = path.with_extension("lrc");
            let txt_path = path.with_extension("txt");
            if lrc_path.exists() {
                fs::read_to_string(&lrc_path).ok()
            } else if txt_path.exists() {
                fs::read_to_string(&txt_path).ok()
            } else {
                None
            }
        }
        // We can't support sync lyrics from ID3v2
        // See more here: https://github.com/Serial-ATA/lofty-rs/discussions/632
        if tag.tag_type() == TagType::Id3v2 {
            if lyrics.is_none() {
                lyrics = tag.get_string(ItemKey::UnsyncLyrics).map(|s| s.to_string())
            }
            lyrics
        } else {
            if lyrics.is_none() {
                lyrics = tag.get_string(ItemKey::Lyrics).map(|s| s.to_string())
            }
            lyrics
        }
    }

    fn get_lyrics(raw: String) -> Vec<(Option<f64>, String)> {
        let lyrics = Lyrics::from_str(&raw).ok();
        if let Some(lyrics) = lyrics {
            return lyrics.to_vec();
        }
        vec![]
    }
}
