use legion::prelude::*;
use skulpin::winit::event::VirtualKeyCode;
use crate::resources::InputResource;
use crate::resources::AppControlResource;

pub fn quit_if_escape_pressed() -> Box<dyn Schedulable> {
    SystemBuilder::new("quit_if_escape_pressed")
        .read_resource::<InputResource>()
        .write_resource::<AppControlResource>()
        .build(|_, _, (input_state, app_control), _| {
            if input_state.is_key_down(VirtualKeyCode::Escape) {
                app_control.enqueue_terminate_process();
            }
        })
}
