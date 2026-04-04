use gtk::{
    gio::{self, prelude::FileExt},
    glib,
};
use log::error;
use mpris_server::{LoopStatus, Metadata, PlaybackStatus, Player, Time};
use relm4::ComponentSender;
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};

use crate::app::{AppModel, AppMsg, RepeatStage};
use crate::backend::track_info::TrackInfo;
use crate::{APP_ID, backend::cover_cache::CoverCache};
#[derive(Debug, Clone)]
pub struct MprisController {
    mpris: Rc<OnceCell<Player>>,
    track: RefCell<Option<TrackInfo>>,
}

impl MprisController {
    pub fn new(sender: &ComponentSender<AppModel>) -> Self {
        let builder = Player::builder(APP_ID)
            .identity("PacTune")
            .desktop_entry(APP_ID)
            .can_raise(true)
            .can_play(true)
            .can_pause(true)
            .can_seek(true)
            .can_go_next(true)
            .can_go_previous(true)
            .can_set_fullscreen(false);

        let mpris = Rc::new(OnceCell::new());

        glib::spawn_future_local(glib::clone!(
            #[weak]
            mpris,
            #[strong]
            sender,
            async move {
                match builder.build().await {
                    Err(err) => error!("Failed to create MPRIS server: {:?}", err),
                    Ok(player) => {
                        setup_signals(sender, &player);
                        let mpris_task = player.run();
                        let _ = mpris.set(player);
                        mpris_task.await;
                    }
                }
            }
        ));

        Self {
            mpris,
            track: RefCell::new(None),
        }
    }

    fn set_playback_status(&self, status: PlaybackStatus) {
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = mpris)]
            self.mpris,
            async move {
                if let Some(mpris) = mpris.get()
                    && let Err(err) = mpris.set_playback_status(status).await
                {
                    error!("Unable to set MPRIS playback status: {err:?}");
                }
            }
        ));
    }

    pub fn set_playing(&self) {
        self.set_playback_status(PlaybackStatus::Playing);
    }

    pub fn set_paused(&self) {
        self.set_playback_status(PlaybackStatus::Paused);
    }

    pub fn set_stopped(&self) {
        self.set_playback_status(PlaybackStatus::Stopped);
    }

    fn update_metadata(&self) {
        let mut metadata = Metadata::new();
        if let Some(track) = self.track.take() {
            metadata.set_artist(Some(vec![track.artist()]));
            metadata.set_title(Some(track.title()));
            metadata.set_album(Some(track.album()));
            let length = Time::from_secs(track.duration() as i64);
            metadata.set_length(Some(length));

            let art_url = track.cover_uuid().and_then(|uuid| {
                let cache = CoverCache::global().lock().unwrap();
                let path = cache.get_cache_path(uuid).unwrap();
                let file = gio::File::for_path(path);

                file.query_info(
                    "standard::type",
                    gio::FileQueryInfoFlags::NONE,
                    gio::Cancellable::NONE,
                )
                .ok()
                .filter(|info| info.file_type() == gio::FileType::Regular)
                .map(|_| file.uri())
            });

            metadata.set_art_url(art_url);

            self.track.replace(Some(track));
        }

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = mpris)]
            self.mpris,
            async move {
                if let Some(mpris) = mpris.get()
                    && let Err(err) = mpris.set_metadata(metadata).await
                {
                    error!("Unable to set MPRIS metadata: {err:?}");
                }
            }
        ));
    }

    pub fn set_track(&self, track: &TrackInfo) {
        self.track.replace(Some(track.clone()));
        self.update_metadata();
    }

    pub fn set_position(&self, position: u64) {
        let pos = Time::from_secs(position as i64);
        if let Some(mpris) = self.mpris.get() {
            mpris.set_position(pos);
        }
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = mpris)]
            self.mpris,
            async move {
                if let Some(mpris) = mpris.get()
                    && let Err(err) = mpris.seeked(pos).await
                {
                    error!("Unable to emit MPRIS Seeked: {err:?}");
                }
            }
        ));
    }
}

fn setup_signals(sender: ComponentSender<AppModel>, mpris: &Player) {
    mpris.connect_play_pause(glib::clone!(
        #[strong]
        sender,
        move |player| {
            match player.playback_status() {
                PlaybackStatus::Paused => sender.input(AppMsg::Play),
                PlaybackStatus::Stopped => sender.input(AppMsg::Stop),
                _ => sender.input(AppMsg::Pause),
            };
        }
    ));

    mpris.connect_play(glib::clone!(
        #[strong]
        sender,
        move |_| sender.input(AppMsg::Play)
    ));

    mpris.connect_stop(glib::clone!(
        #[strong]
        sender,
        move |_| sender.input(AppMsg::Stop)
    ));

    mpris.connect_pause(glib::clone!(
        #[strong]
        sender,
        move |_| sender.input(AppMsg::Pause)
    ));

    mpris.connect_previous(glib::clone!(
        #[strong]
        sender,
        move |_| sender.input(AppMsg::TrackPrevious)
    ));

    mpris.connect_next(glib::clone!(
        #[strong]
        sender,
        move |_| sender.input(AppMsg::TrackNext)
    ));

    mpris.connect_seek(glib::clone!(
        #[strong]
        sender,
        move |_, offset| {
            let offset = offset.as_secs();
            sender.input(AppMsg::EndGetPosition(offset as f64))
        }
    ));

    mpris.connect_set_loop_status(glib::clone!(
        #[strong]
        sender,
        move |_, status| {
            let mode = match status {
                LoopStatus::None => RepeatStage::NotRepeat,
                LoopStatus::Track => RepeatStage::RepeatTrack,
                LoopStatus::Playlist => RepeatStage::RepeatPlaylist,
            };
            sender.input(AppMsg::SetRepeatStage(mode));
        }
    ));
}
