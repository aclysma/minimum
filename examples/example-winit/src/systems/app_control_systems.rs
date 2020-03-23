use legion::prelude::*;
use skulpin::winit::event::VirtualKeyCode;
use minimum::resources::InputResource;
use minimum::resources::AppControlResource;
use minimum_winit::input::WinitKeyboardKey;

pub fn quit_if_escape_pressed() -> Box<dyn Schedulable> {
    SystemBuilder::new("quit_if_escape_pressed")
        .read_resource::<InputResource>()
        .write_resource::<AppControlResource>()
        .build(|_, _, (input_state, app_control), _| {
            if input_state.is_key_down(WinitKeyboardKey::new(VirtualKeyCode::Escape).into()) {
                app_control.enqueue_terminate_process();
            }
        })
}
