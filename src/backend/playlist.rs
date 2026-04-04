use crate::{app::RepeatStage, backend::track_info::TrackInfo};
use std::sync::Arc;

#[derive(Debug)]
pub struct Playlist {
    pub main_list: Vec<Arc<TrackInfo>>,
    pub current_track: usize,
}

impl Playlist {
    pub fn new(main_list: Vec<Arc<TrackInfo>>, current_track: usize) -> Self {
        Playlist {
            main_list,
            current_track,
        }
    }

    pub fn track_next(&mut self, stage: &RepeatStage) {
        let index_next = self.current_track + 1;
        if index_next < self.main_list.len() {
            self.set_current_track(index_next);
        } else if stage == &RepeatStage::RepeatPlaylist {
            self.set_current_track(0)
        }
    }

    pub fn track_previous(&mut self, stage: &RepeatStage) {
        let index_next = self.current_track as i32 - 1;
        if index_next >= 0 {
            self.current_track = index_next as usize;
        } else if stage == &RepeatStage::RepeatPlaylist {
            self.set_current_track(self.main_list.len() - 1)
        }
    }

    pub fn get_track(&self) -> Option<&TrackInfo> {
        // Avoid rust panic, if .json file in the /state was changed
        if self.current_track <= self.main_list.len() {
            return Some(&self.main_list[self.current_track]);
        }
        None
    }

    pub fn set_current_track(&mut self, new_index: usize) {
        self.current_track = new_index
    }
}
