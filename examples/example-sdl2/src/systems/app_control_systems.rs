use legion::*;
use sdl2::keyboard::Keycode;
use minimum::resources::InputResource;
use minimum::resources::AppControlResource;
use minimum_sdl2::input::Sdl2KeyboardKey;

pub fn quit_if_escape_pressed(schedule: &mut legion::systems::Builder) {
    schedule.add_system(
        SystemBuilder::new("quit_if_escape_pressed")
            .read_resource::<InputResource>()
            .write_resource::<AppControlResource>()
            .build(|_, _, (input_state, app_control), _| {
                if input_state.is_key_down(Sdl2KeyboardKey::new(Keycode::Escape).into()) {
                    app_control.enqueue_terminate_process();
                }
            }),
    );
}
