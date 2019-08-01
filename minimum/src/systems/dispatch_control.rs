pub struct DispatchControl {
    next_frame_context_flags: usize,
    should_terminate: bool,
}

impl DispatchControl {
    pub fn new(context_flags: usize) -> Self {
        DispatchControl {
            next_frame_context_flags: context_flags,
            should_terminate: false,
        }
    }

    pub fn next_frame_context_flags(&self) -> usize {
        self.next_frame_context_flags
    }

    pub fn next_frame_context_flags_mut(&mut self) -> &mut usize {
        &mut self.next_frame_context_flags
    }

    pub fn end_game_loop(&mut self) {
        self.should_terminate = true;
    }

    pub fn should_terminate(&self) -> bool {
        self.should_terminate
    }
}
