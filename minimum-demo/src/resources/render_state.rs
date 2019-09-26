pub struct RenderState {
    backbuffer_count: u32,
}

// UI space: pixels, top-left: (0, 0), bottom-right: (window width in pixels, window height in pixels)
// Raw space: top-left: (-1, -1), bottom-right: (1, 1)
// world space: x positive to the right, y positive going up. width/values depend on camera
// screen space: top-left: (0, 600), bottom-right: (+x, 0) where +x is 600 * screen ratio (i.e. 1066 = ((16/9 * 600) for a 16:9 screen)
impl RenderState {
    //TODO: Find some alternative that prevents this from having to ever be in an invalid state
    pub fn empty() -> Self {
        RenderState {
            backbuffer_count: 0,
        }
    }

    pub fn init(
        &mut self,
        backbuffer_count: u32,
    ) {
        self.backbuffer_count = backbuffer_count;
    }

    pub fn backbuffer_count(&self) -> u32 {
        self.backbuffer_count
    }

}
