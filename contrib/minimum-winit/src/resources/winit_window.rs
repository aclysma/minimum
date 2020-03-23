use winit::window::Window;
use winit::dpi::PhysicalSize;

pub struct WinitWindowResource {
    inner_size: PhysicalSize<u32>,
}

impl WinitWindowResource {
    pub fn new(window: &Window) -> Self {
        WinitWindowResource {
            inner_size: window.inner_size(),
        }
    }

    pub fn update(
        &mut self,
        window: &Window,
    ) {
        self.inner_size = window.inner_size();
    }

    pub fn inner_size(&self) -> PhysicalSize<u32> {
        self.inner_size
    }
}
