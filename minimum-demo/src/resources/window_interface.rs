pub enum WindowUserEvent {
    Terminate,
}

pub struct WindowInterface {
    pub event_rx: std::sync::Mutex<std::sync::mpsc::Receiver<winit::event::Event<WindowUserEvent>>>,
    pub event_loop_proxy: winit::event_loop::EventLoopProxy<WindowUserEvent>,
}

impl WindowInterface {
    pub fn new(
        event_rx: std::sync::Mutex<std::sync::mpsc::Receiver<winit::event::Event<WindowUserEvent>>>,
        event_loop_proxy: winit::event_loop::EventLoopProxy<WindowUserEvent>,
    ) -> Self {
        WindowInterface {
            event_rx,
            event_loop_proxy,
        }
    }
}
