use std::ops::{Deref, DerefMut};
use skulpin::app::InputState;
use skulpin::app::PhysicalSize;
use crate::resources::ViewportResource;
use crate::math::winit_position_to_glam;
use skulpin::app::VirtualKeyCode;
use skulpin::app::MouseButton;
use skulpin::app::MouseScrollDelta;

// Keep track of a drag state so that we can track world space movement on drag. There are some
// floating point precision issues and it's better to deal with it once here than everywhere
#[derive(Clone, Debug)]
pub struct MouseDragState {
    pub begin_position: glam::Vec2,
    pub end_position: glam::Vec2,
    pub previous_frame_delta: glam::Vec2,
    pub accumulated_frame_delta: glam::Vec2,
    pub world_space_begin_position: glam::Vec2,
    pub world_space_end_position: glam::Vec2,
    pub world_space_previous_frame_delta: glam::Vec2,
    pub world_space_accumulated_frame_delta: glam::Vec2,
    pub world_scale_previous_frame_delta: glam::Vec2,
    pub world_scale_accumulated_frame_delta: glam::Vec2,
}

// For now just wrap the input helper that skulpin provides
pub struct InputResource {
    input_state: InputState,
    mouse_drag_in_progress: [Option<MouseDragState>; InputState::MOUSE_BUTTON_COUNT],
    mouse_drag_just_finished: [Option<MouseDragState>; InputState::MOUSE_BUTTON_COUNT],
}

impl InputResource {
    pub fn new(input_state: InputState) -> Self {
        InputResource {
            input_state,
            mouse_drag_in_progress: Default::default(),
            mouse_drag_just_finished: Default::default(),
        }
    }

    pub fn input_state(&self) -> &InputState {
        &self.input_state
    }

    pub fn input_state_mut(&mut self) -> &mut InputState {
        &mut self.input_state
    }

    pub fn window_size(&self) -> PhysicalSize<u32> {
        self.input_state.window_size()
    }

    /// Returns true if the given key is down
    pub fn is_key_down(
        &self,
        key: VirtualKeyCode,
    ) -> bool {
        self.input_state.is_key_down(key)
    }

    /// Returns true if the key went down during this frame
    pub fn is_key_just_down(
        &self,
        key: VirtualKeyCode,
    ) -> bool {
        self.input_state.is_key_just_down(key)
    }

    /// Returns true if the key went up during this frame
    pub fn is_key_just_up(
        &self,
        key: VirtualKeyCode,
    ) -> bool {
        self.input_state.is_key_just_up(key)
    }

    /// Get the current mouse position
    pub fn mouse_position(&self) -> glam::Vec2 {
        winit_position_to_glam(self.input_state.mouse_position())
    }

    /// Get the scroll delta from the current frame
    pub fn mouse_wheel_delta(&self) -> MouseScrollDelta {
        self.input_state.mouse_wheel_delta()
    }

    /// Returns true if the given button is down
    pub fn is_mouse_down(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        self.input_state.is_mouse_down(mouse_button)
    }

    /// Returns true if the button went down during this frame
    pub fn is_mouse_just_down(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        self.input_state.is_mouse_just_down(mouse_button)
    }

    /// Returns the position the mouse just went down at, otherwise returns None
    pub fn mouse_just_down_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<glam::Vec2> {
        self.input_state
            .mouse_just_down_position(mouse_button)
            .map(|p| winit_position_to_glam(p))
    }

    /// Returns true if the button went up during this frame
    pub fn is_mouse_just_up(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        self.input_state.is_mouse_just_up(mouse_button)
    }

    /// Returns the position the mouse just went up at, otherwise returns None
    pub fn mouse_just_up_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<glam::Vec2> {
        self.input_state
            .mouse_just_up_position(mouse_button)
            .map(|p| winit_position_to_glam(p))
    }

    /// Returns true if the button was just clicked. "Clicked" means the button went down and came
    /// back up without being moved much. If it was moved, it would be considered a drag.
    pub fn is_mouse_button_just_clicked(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        self.input_state.is_mouse_button_just_clicked(mouse_button)
    }

    /// Returns the position the button was just clicked at, otherwise None. "Clicked" means the
    /// button went down and came back up without being moved much. If it was moved, it would be
    /// considered a drag.
    pub fn mouse_button_just_clicked_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<glam::Vec2> {
        self.input_state
            .mouse_button_just_clicked_position(mouse_button)
            .map(|p| winit_position_to_glam(p))
    }

    /// Returns the position the button went down at previously. This could have been some time ago.
    pub fn mouse_button_went_down_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<glam::Vec2> {
        self.input_state
            .mouse_button_went_down_position(mouse_button)
            .map(|p| winit_position_to_glam(p))
    }

    /// Returns the position the button went up at previously. This could have been some time ago.
    pub fn mouse_button_went_up_position(
        &self,
        mouse_button: MouseButton,
    ) -> Option<glam::Vec2> {
        self.input_state
            .mouse_button_went_up_position(mouse_button)
            .map(|p| winit_position_to_glam(p))
    }

    /// Return true if the mouse is being dragged. (A drag means the button went down and mouse
    /// moved, but button hasn't come back up yet)
    pub fn is_mouse_drag_in_progress(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        if let Some(index) = InputState::mouse_button_to_index(mouse_button) {
            self.mouse_drag_in_progress[index].is_some()
        } else {
            false
        }
    }

    /// Returns the mouse drag state if a drag is in process, otherwise None.
    pub fn mouse_drag_in_progress(
        &self,
        mouse_button: MouseButton,
    ) -> Option<&MouseDragState> {
        if let Some(index) = InputState::mouse_button_to_index(mouse_button) {
            self.mouse_drag_in_progress[index].as_ref()
        } else {
            None
        }
    }

    /// Return true if a mouse drag completed in the previous frame, otherwise false
    pub fn is_mouse_drag_just_finished(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        if let Some(index) = InputState::mouse_button_to_index(mouse_button) {
            self.mouse_drag_just_finished[index].is_some()
        } else {
            false
        }
    }

    /// Returns information about a mouse drag if it just completed, otherwise None
    pub fn mouse_drag_just_finished(
        &self,
        mouse_button: MouseButton,
    ) -> Option<&MouseDragState> {
        if let Some(index) = InputState::mouse_button_to_index(mouse_button) {
            self.mouse_drag_just_finished[index].as_ref()
        } else {
            None
        }
    }

    fn create_drag_state(
        viewport: &ViewportResource,
        new_drag_state: skulpin::app::MouseDragState,
        old_drag_state: Option<&MouseDragState>,
    ) -> MouseDragState {
        let begin_position = winit_position_to_glam(new_drag_state.begin_position);
        let end_position = winit_position_to_glam(new_drag_state.end_position);
        let previous_frame_delta = winit_position_to_glam(new_drag_state.previous_frame_delta);
        let accumulated_frame_delta =
            winit_position_to_glam(new_drag_state.accumulated_frame_delta);

        if let Some(old_drag_state) = old_drag_state {
            //
            // Case for a drag in progress from a prior frame ending on this frame
            //

            // This is where the cursor is now
            let world_space_end_position = viewport.ui_space_to_world_space(end_position);

            // This is the math for if we want deltas to be based on begin/end position rather than mouse movement at world scale
            let world_space_delta = world_space_end_position
                - (old_drag_state.world_space_begin_position
                    + old_drag_state.world_space_accumulated_frame_delta);

            // Determine what delta is required to reach the end position, given our original begin position and accumulated deltas
            // so far
            let total_ui_space_delta = end_position - begin_position;
            let total_world_space_delta =
                viewport.ui_space_delta_to_world_space_delta(total_ui_space_delta);
            let world_scale_delta =
                total_world_space_delta - old_drag_state.world_scale_accumulated_frame_delta;

            MouseDragState {
                begin_position,
                end_position,
                previous_frame_delta,
                accumulated_frame_delta,
                world_space_begin_position: old_drag_state.world_space_begin_position,
                world_space_end_position,
                world_space_previous_frame_delta: world_space_delta,
                world_space_accumulated_frame_delta: old_drag_state
                    .world_space_accumulated_frame_delta
                    + world_space_delta,
                world_scale_previous_frame_delta: world_scale_delta,
                world_scale_accumulated_frame_delta: old_drag_state
                    .world_scale_accumulated_frame_delta
                    + world_scale_delta,
            }
        } else {
            //
            // Case for a drag starting and completing on a single frame
            //
            let world_space_begin_position = viewport.ui_space_to_world_space(begin_position);
            let world_space_end_position = viewport.ui_space_to_world_space(end_position);
            let delta = world_space_end_position - world_space_begin_position;

            MouseDragState {
                begin_position,
                end_position,
                previous_frame_delta,
                accumulated_frame_delta,
                world_space_begin_position,
                world_space_end_position,
                world_space_previous_frame_delta: delta,
                world_space_accumulated_frame_delta: delta,
                world_scale_previous_frame_delta: delta,
                world_scale_accumulated_frame_delta: delta,
            }
        }
    }

    pub fn update(
        &mut self,
        viewport: &ViewportResource,
    ) {
        for button_index in 0..InputState::MOUSE_BUTTON_COUNT {
            let button = InputState::mouse_index_to_button(button_index).unwrap();

            //TODO: Skulpin's API could use an improvement here, not great to have to check two functions
            // could return an enum
            if let Some(new_drag_state) = self.input_state.mouse_drag_in_progress(button) {
                self.mouse_drag_in_progress[button_index] = Some(Self::create_drag_state(
                    viewport,
                    new_drag_state,
                    self.mouse_drag_in_progress[button_index].as_ref(),
                ));
                self.mouse_drag_just_finished[button_index] = None;
            } else if let Some(new_drag_state) = self.input_state.mouse_drag_just_finished(button) {
                self.mouse_drag_just_finished[button_index] = Some(Self::create_drag_state(
                    viewport,
                    new_drag_state,
                    self.mouse_drag_in_progress[button_index].as_ref(),
                ));
                self.mouse_drag_in_progress[button_index] = None;
            } else {
                self.mouse_drag_just_finished[button_index] = None;
                self.mouse_drag_in_progress[button_index] = None;
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.input_state.end_frame();
    }
}
