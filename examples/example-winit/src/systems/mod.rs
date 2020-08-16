pub use example_shared::systems::*;

mod app_control_systems;
pub use app_control_systems::quit_if_escape_pressed;

mod draw_systems;
pub use draw_systems::draw;

use minimum::systems::*;

use legion::*;

use minimum::editor::resources::EditorMode;
use minimum_nphysics2d::systems::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ScheduleCriteria {
    is_simulation_paused: bool,
    editor_mode: EditorMode,
}

impl ScheduleCriteria {
    pub fn new(
        is_simulation_paused: bool,
        editor_mode: EditorMode,
    ) -> Self {
        ScheduleCriteria {
            is_simulation_paused,
            editor_mode,
        }
    }
}

struct ScheduleBuilder<'a> {
    criteria: &'a ScheduleCriteria,
    schedule: legion::systems::Builder,
}

impl<'a> ScheduleBuilder<'a> {
    fn new(criteria: &'a ScheduleCriteria) -> Self {
        ScheduleBuilder::<'a> {
            criteria,
            schedule: Default::default(),
        }
    }

    fn build(mut self) -> Schedule {
        self.schedule.build()
    }

    fn always<F>(
        mut self,
        f: F,
    ) -> Self
        where
            F: Fn(&mut legion::systems::Builder),
    {
        (f)(&mut self.schedule);
        self
    }

    fn editor_only<F>(
        mut self,
        f: F,
    ) -> Self
        where
            F: Fn(&mut legion::systems::Builder),
    {
        if self.criteria.editor_mode == EditorMode::Active {
            (f)(&mut self.schedule);
        }

        self
    }

    fn simulation_unpaused_only<F>(
        mut self,
        f: F,
    ) -> Self
        where
            F: Fn(&mut legion::systems::Builder),
    {
        if !self.criteria.is_simulation_paused {
            (f)(&mut self.schedule);
        }

        self
    }

    fn always_thread_local<F: FnMut(&mut World, &mut Resources) + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.schedule.add_thread_local_fn(f);
        self
    }

    fn flush(mut self) -> Self {
        self.schedule.flush();
        self
    }
}

pub fn create_update_schedule(criteria: &ScheduleCriteria) -> Schedule {
    use minimum::editor::systems::*;

    ScheduleBuilder::new(criteria)
        .always(update_input_resource)
        .always(advance_time)
        .always(quit_if_escape_pressed)
        .always_thread_local(update_asset_manager)
        .always(update_fps_text)
        .always(update_physics)
        .simulation_unpaused_only(read_from_physics)
        // --- Editor stuff here ---
        // Prepare to handle editor input
        .always_thread_local(editor_refresh_selection_world)
        // Editor input
        .always_thread_local(reload_editor_state_if_file_changed)
        .always(editor_keybinds)
        .always(editor_mouse_input)
        .always(editor_update_editor_draw)
        .always(editor_gizmos)
        .always(editor_handle_selection)
        .always(editor_imgui_menu)
        .always(editor_entity_list_window)
        .always_thread_local(editor_inspector_window)
        // Editor processing
        .always_thread_local(editor_process_edit_diffs)
        .always_thread_local(editor_process_selection_ops)
        .always_thread_local(editor_process_editor_ops)
        // Editor output
        .always(draw_selection_shapes)
        // --- End editor stuff ---
        .always(input_reset_for_next_frame)
        .build()
}

pub fn create_draw_schedule(criteria: &ScheduleCriteria) -> Schedule {
    ScheduleBuilder::new(criteria).always(draw).build()
}
