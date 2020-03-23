use sdl2::video::Window;

#[derive(Copy, Clone)]
pub struct Sdl2WindowSize {
    pub width: u32,
    pub height: u32,
}

impl From<(u32, u32)> for Sdl2WindowSize {
    fn from(size: (u32, u32)) -> Self {
        Sdl2WindowSize {
            width: size.0,
            height: size.1,
        }
    }
}

pub struct Sdl2WindowResource {
    drawable_size: Sdl2WindowSize,
}

impl Sdl2WindowResource {
    pub fn new(window: &Window) -> Self {
        Sdl2WindowResource {
            drawable_size: window.drawable_size().into(),
        }
    }

    pub fn update(
        &mut self,
        window: &Window,
    ) {
        self.drawable_size = window.drawable_size().into();
    }

    pub fn drawable_size(&self) -> Sdl2WindowSize {
        self.drawable_size
    }
}
