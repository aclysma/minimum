use legion::*;

use crate::resources::TimeResource;

pub fn advance_time(schedule: &mut legion::systems::Builder) {
    schedule.add_system(
        SystemBuilder::new("advance_time")
            .write_resource::<TimeResource>()
            .build(|_, _, time_resource, _| {
                time_resource.process_time_ops();
                time_resource.advance_time();

                let now = time_resource.time_state.current_instant();
                if time_resource
                    .log_fps_event
                    .try_take_event(now, std::time::Duration::from_secs(1))
                {
                    log::debug!("fps: {}", time_resource.time_state.updates_per_second());
                }
            }),
    );
}
