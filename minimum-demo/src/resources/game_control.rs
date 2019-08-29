use std::path::PathBuf;
use crate::PlayMode;

pub struct GameControl {
    load_level: Option<PathBuf>,
    save_level: Option<PathBuf>,
    change_play_mode: Option<PlayMode>,
    reset_level: bool,
    terminate_process: bool,
}

//TODO: Rename to FrameworkContorl?
impl GameControl {
    pub fn new() -> Self {
        GameControl {
            load_level: None,
            save_level: None,
            change_play_mode: None,
            reset_level: false,
            terminate_process: false,
        }
    }

    //
    // Load level from file
    //
    pub fn set_load_level(&mut self, path: PathBuf) {
        self.load_level = Some(path);
    }

    pub fn has_load_level(&self) -> bool {
        self.load_level.is_some()
    }

    pub fn take_load_level(&mut self) -> Option<PathBuf> {
        if self.load_level.is_some() {
            let value = self.load_level.clone();
            self.load_level = None;
            value
        } else {
            None
        }
    }

    //
    // Save level to file
    //
    pub fn set_save_level(&mut self, path: PathBuf) {
        self.save_level = Some(path);
    }

    pub fn has_save_level(&self) -> bool {
        self.save_level.is_some()
    }

    pub fn take_save_level(&mut self) -> Option<PathBuf> {
        if self.save_level.is_some() {
            let value = self.save_level.clone();
            self.save_level = None;
            value
        } else {
            None
        }
    }

    //
    // change_play_mode
    //
    pub fn set_change_play_mode(&mut self, play_mode: PlayMode) {
        self.change_play_mode = Some(play_mode);
    }

    pub fn has_change_play_mode(&self) -> bool {
        self.change_play_mode.is_some()
    }

    pub fn take_change_play_mode(&mut self) -> Option<PlayMode> {
        if self.change_play_mode.is_some() {
            let value = self.change_play_mode.clone();
            self.change_play_mode = None;
            value
        } else {
            None
        }
    }

    //
    // Reset level
    //
    pub fn set_reset_level(&mut self) {
        self.reset_level = true;
    }

    pub fn has_reset_level(&self) -> bool {
        self.reset_level
    }

    pub fn take_reset_level(&mut self) -> bool {
        if self.reset_level {
            self.reset_level = false;
            true
        } else {
            false
        }
    }

    //
    // Terminate process
    //
    pub fn set_terminate_process(&mut self) {
        self.terminate_process = true;
    }

    pub fn has_terminate_process(&self) -> bool {
        self.terminate_process
    }

    pub fn take_terminate_process(&mut self) -> bool {
        if self.terminate_process {
            self.terminate_process = false;
            true
        } else {
            false
        }
    }
}
