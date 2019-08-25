use std::path::PathBuf;

pub struct GameControl {
    load_level: Option<PathBuf>,
    save_level: Option<PathBuf>,
    terminate_process: bool,
}

impl GameControl {
    pub fn new() -> Self {
        GameControl {
            load_level: None,
            save_level: None,
            terminate_process: false,
        }
    }

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

    pub fn terminate_process(&self) -> bool {
        self.terminate_process
    }

    pub fn set_terminate_process(&mut self) {
        self.terminate_process = true;
    }
}
