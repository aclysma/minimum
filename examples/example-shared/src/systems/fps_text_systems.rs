use legion::prelude::*;

use crate::resources::FpsTextResource;
use minimum::resources::TimeResource;

pub fn update_fps_text() -> Box<dyn Schedulable> {
    SystemBuilder::new("update fps text")
        .read_resource::<TimeResource>()
        .write_resource::<FpsTextResource>()
        .build(|_, _, (time_resource, fps_text), _| {
            let now = time_resource.time_state.current_instant();
            //
            // Update FPS once a second
            //
            let update_text_string = match fps_text.last_fps_text_change {
                Some(last_update_instant) => (now - last_update_instant).as_secs_f32() >= 1.0,
                None => true,
            };

            // Refresh FPS text
            if update_text_string {
                let fps = time_resource.time_state.updates_per_second();
                fps_text.fps_text = format!("Fps: {:.1}", fps);
                fps_text.last_fps_text_change = Some(now);
            }
        })
}
