use crate::app::AppModel;
use crate::app::AppMsg;
use crate::app::RepeatStage;
use crate::backend::mpris::MprisController;
use crate::backend::playlist::Playlist;
use crate::backend::track_info::TrackInfo;
use gst::glib;
use gst::prelude::*;
use gst_play::*;
use log::{debug, error};
use relm4::ComponentSender;
use std::sync::Arc;

#[derive(Debug)]
pub struct PlayerState {
    playlist: Playlist,
    player: gst_play::Play,
    sender: ComponentSender<AppModel>,
    mpris: MprisController,
}

impl PlayerState {
    pub fn new(playlist: Vec<Arc<TrackInfo>>, sender: ComponentSender<AppModel>) -> Self {
        let player = {
            let player_state = gst_play::Play::default();
            player_state.set_video_track_enabled(false);
            let mut config = player_state.config();
            config.set_position_update_interval(100);
            player_state.set_config(config).unwrap();
            player_state
        };

        let player_state = PlayerState {
            playlist: Playlist::new(playlist, 0),
            player,
            mpris: MprisController::new(&sender),
            sender,
        };

        player_state.signal_handler();

        player_state
    }

    pub fn is_playing(&self) -> bool {
        self.player.pipeline().current_state() == gst::State::Playing
    }

    pub fn clear(&mut self) {
        self.player.stop();
        // We need save current_track for restore playlist
        let current_track = self.playlist.current_track;
        self.playlist = Playlist::new(Vec::new(), current_track);
    }

    pub fn play_track(&mut self) {
        if let Some(uri) = &self.make_uri() {
            self.player.set_uri(Some(uri));
            self.player.play();
            self.mpris.set_playing();
        } else {
            self.sender
                .input(AppMsg::AdwToastBuild("Error to play track".into()))
        }
    }

    pub fn stop_track(&mut self) {
        if let Some(uri) = &self.make_uri() {
            self.player.set_uri(Some(uri));
            self.player.stop();
            self.mpris.set_stopped();
        } else {
            self.sender
                .input(AppMsg::AdwToastBuild("Error to set track".into()))
        }
    }

    pub fn initial_track(&mut self) {
        if let Some(uri) = &self.make_uri() {
            self.player.set_uri(Some(uri));
            self.mpris.set_paused();
            self.sender.input(AppMsg::UpdateUiInitialTrack);
        } else {
            self.sender
                .input(AppMsg::AdwToastBuild("Error to set track".into()))
        }
    }

    pub fn play(&mut self) {
        self.player.play();
        self.mpris.set_playing();
    }

    pub fn pause(&mut self) {
        self.player.pause();
        self.mpris.set_paused();
    }

    pub fn stop(&mut self) {
        self.player.stop();
        self.mpris.set_stopped();
    }

    pub fn set_volume(&mut self, volume: f64) {
        let linear_volume = gst_audio::StreamVolume::convert_volume(
            gst_audio::StreamVolumeFormat::Cubic,
            gst_audio::StreamVolumeFormat::Linear,
            volume,
        );
        debug!("Setting volume to: {}", &linear_volume);
        self.player.set_volume(linear_volume);
    }

    pub fn track_next(&mut self, stage: &RepeatStage) {
        self.playlist.track_next(stage);
        self.sender.input(AppMsg::UpdateUiSimpleTrack);
        self.play_track();
    }

    pub fn track_previous(&mut self, stage: &RepeatStage) {
        self.playlist.track_previous(stage);
        self.sender.input(AppMsg::UpdateUiSimpleTrack);
        self.play_track();
    }
    fn signal_handler(&self) {
        let bus = self.player.message_bus();
        bus.set_sync_handler(glib::clone!(
            #[strong(rename_to=sender)]
            self.sender,
            move |_bus, msg| {
                let Ok(play_msg) = gst_play::PlayMessage::parse(msg) else {
                    return gst::BusSyncReply::Drop;
                };
                match play_msg {
                    PlayMessage::EndOfStream(_) => {
                        if let Err(e) = sender.input_sender().send(AppMsg::NotifyTrackEnd) {
                            error!("Failed to send RepeatOrNextTrack: {:?}", e);
                        }
                    }
                    PlayMessage::PositionUpdated(position) => {
                        if let Some(position) = position.position()
                            && let Err(e) = sender
                                .input_sender()
                                .send(AppMsg::UpdatePosition(position.seconds()))
                        {
                            error!("Failed to send position: {:?}", e);
                        }
                    }
                    PlayMessage::VolumeChanged(message) => {
                        let volume = gst_audio::StreamVolume::convert_volume(
                            gst_audio::StreamVolumeFormat::Linear,
                            gst_audio::StreamVolumeFormat::Cubic,
                            message.volume(),
                        );
                        if let Err(e) = sender.input_sender().send(AppMsg::ChangeVolume(volume)) {
                            error!("Failed to send VolumeChanged({volume}): {:?}", e);
                        }
                    }

                    PlayMessage::Error(_msg) => {
                        if let Err(e) = sender
                            .input_sender()
                            .send(AppMsg::AdwToastBuild("Gst Error".into()))
                        {
                            error!("Failed to send AdwToastBuild: {:?}", e);
                        }
                    }
                    _ => {}
                }
                gst::BusSyncReply::Drop
            }
        ));
    }

    pub fn seek_position(&self, position: f64) {
        let get_pos = gst::ClockTime::from_seconds(position as u64);
        self.player.seek(get_pos);
        self.mpris.set_position(position as u64);
    }

    fn make_uri(&mut self) -> Option<String> {
        let music_file = &self.playlist.get_track();
        if let Some(music_file) = music_file {
            let path = &music_file.path();
            self.mpris.set_track(music_file);
            return Some(glib::filename_to_uri(path, None).unwrap().to_string());
        }
        None
    }

    pub fn set_current_track(&mut self, new_index: usize) {
        self.playlist.set_current_track(new_index);
    }

    pub fn current_track(&self) -> usize {
        self.playlist.current_track
    }

    pub fn len(&self) -> usize {
        self.playlist.main_list.len()
    }

    pub fn get_track(&self) -> Option<&TrackInfo> {
        self.playlist.get_track()
    }

    pub fn extend(&mut self, vec: Vec<Arc<TrackInfo>>) {
        self.playlist.main_list.extend(vec)
    }

    pub fn tracks_for_album(&self, album_name: &str) -> Vec<(usize, Arc<TrackInfo>)> {
        self.playlist
            .main_list
            .iter()
            .enumerate()
            .filter(|(_, t)| t.album() == album_name)
            .map(|(i, t)| (i, Arc::clone(t)))
            .collect()
    }
}
