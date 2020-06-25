use minimum_game::resources::DebugDrawResource;
use minimum_game::resources::ViewportResource;

use minimum_game::input::InputState;
use minimum_game::input::MouseButton;

fn distance_to_segment_sq(
    test_point: glam::Vec2,
    p0: glam::Vec2,
    p1: glam::Vec2,
) -> f32 {
    let p0_to_p1 = p1 - p0;

    // Early out in case of extremely short segment, get distance to midpoint
    if p0_to_p1.length_squared() < 0.01 {
        let midpoint = p0 + (p0_to_p1 / 2.0);
        return (test_point - midpoint).length_squared();
    }

    // Get "tangent" and "normal" of the segment
    let tangent = p0_to_p1.normalize();
    let normal = glam::Vec2::new(tangent.y(), -tangent.x());

    // distance to the infinite line described by the points
    let p0_to_test_point = test_point - p0;

    // find closest point to the line
    let distance_along_segment = glam::Vec2::dot(tangent, p0_to_test_point);
    if distance_along_segment <= 0.0 {
        // early out, test_point is closer to p0 than any other part of the line
        return (test_point - p0).length_squared();
    }

    let fraction_along_segment =
        (distance_along_segment * distance_along_segment) / p0_to_p1.length_squared();
    if fraction_along_segment >= 1.0 {
        // test_point is closer to p1 than any other part of the line
        (test_point - p1).length_squared()
    } else {
        // the closest point on the segment to test_point is between p0 and p1
        let distance_to_line = glam::Vec2::dot(normal, p0_to_test_point);
        f32::abs(distance_to_line * distance_to_line)
    }
}

fn distance_to_circle_outline_sq(
    test_point: glam::Vec2,
    center: glam::Vec2,
    radius: f32,
) -> f32 {
    ((test_point - center).length_squared() - (radius * radius)).abs()
}

struct Line {
    p0: glam::Vec2,
    p1: glam::Vec2,
}

struct CircleOutline {
    center: glam::Vec2,
    radius: f32,
}

enum Shape {
    Line(Line),
    CircleOutline(CircleOutline),
}

struct ShapeWithId {
    id: String,
    shape: Shape,
}

impl ShapeWithId {
    fn new_line(
        id: String,
        p0: glam::Vec2,
        p1: glam::Vec2,
    ) -> Self {
        ShapeWithId {
            id,
            shape: Shape::Line(Line { p0, p1 }),
        }
    }

    fn new_circle_outline(
        id: String,
        center: glam::Vec2,
        radius: f32,
    ) -> Self {
        ShapeWithId {
            id,
            shape: Shape::CircleOutline(CircleOutline { center, radius }),
        }
    }
}

struct ClosestShapeIdDistance {
    id: String,
    distance_sq: f32,
}

struct ClosestShapeIndexDistance {
    index: usize,
    distance_sq: f32,
}

#[derive(Clone, Debug)]
pub struct EditorShapeClickedState {
    pub click_position: glam::Vec2,
    pub shape_id: String,
}

#[derive(Clone, Debug)]
pub struct EditorShapeDragState {
    pub begin_position: glam::Vec2,
    pub end_position: glam::Vec2,
    pub previous_frame_delta: glam::Vec2,
    pub accumulated_frame_delta: glam::Vec2,
    pub world_space_begin_position: glam::Vec2,
    pub world_space_end_position: glam::Vec2,
    pub world_space_previous_frame_delta: glam::Vec2,
    pub world_space_accumulated_frame_delta: glam::Vec2,
    pub shape_id: String,
}

const MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE_SQ: f32 = 30.0 * 30.0;

//TODO: Rename to EditorShapeDrawResource.. or maybe gizmo?
//TODO: How does this interact with the select/inspect registry?
pub struct EditorDrawResource {
    shapes: Vec<ShapeWithId>,
    shape_just_clicked: [Option<EditorShapeClickedState>; InputState::MOUSE_BUTTON_COUNT as usize],
    shape_drag_in_progress: [Option<EditorShapeDragState>; InputState::MOUSE_BUTTON_COUNT as usize],
    shape_drag_just_finished:
        [Option<EditorShapeDragState>; InputState::MOUSE_BUTTON_COUNT as usize],
    mouse_is_down_on_shape:
        [Option<EditorShapeClickedState>; InputState::MOUSE_BUTTON_COUNT as usize],
    shape_last_interacted: String,
    closest_shape_to_mouse: ClosestShapeIdDistance,
}

impl EditorDrawResource {
    pub fn new() -> Self {
        EditorDrawResource {
            shapes: vec![],
            shape_just_clicked: Default::default(),
            shape_drag_in_progress: Default::default(),
            shape_drag_just_finished: Default::default(),
            mouse_is_down_on_shape: Default::default(),
            shape_last_interacted: "".to_string(),
            closest_shape_to_mouse: ClosestShapeIdDistance {
                id: "".to_string(),
                distance_sq: std::f32::MAX,
            },
        }
    }

    // Input here is world space
    pub fn add_line(
        &mut self,
        id: &str,
        debug_draw: &mut DebugDrawResource,
        p0: glam::Vec2,
        p1: glam::Vec2,
        mut color: glam::Vec4,
    ) {
        if self.closest_shape_to_mouse.id == id
            && self.closest_shape_to_mouse.distance_sq < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE_SQ
        {
            color = glam::vec4(1.0, 0.0, 0.0, 1.0);
        }

        debug_draw.add_line(p0, p1, color);
        self.shapes
            .push(ShapeWithId::new_line(id.to_string(), p0, p1));
    }

    // Input here is world space
    pub fn add_circle_outline(
        &mut self,
        id: &str,
        debug_draw: &mut DebugDrawResource,
        center: glam::Vec2,
        radius: f32,
        mut color: glam::Vec4,
    ) {
        if self.closest_shape_to_mouse.id == id
            && self.closest_shape_to_mouse.distance_sq < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE_SQ
        {
            color = glam::vec4(1.0, 0.0, 0.0, 1.0);
        }

        debug_draw.add_circle(center, radius, color);
        self.shapes.push(ShapeWithId::new_circle_outline(
            id.to_string(),
            center,
            radius,
        ));
    }

    // test_position is ui space, the shapes we keep are in world space
    //TODO: Consider converting to ui space immediately
    fn get_closest_shape(
        &self,
        test_position: glam::Vec2,
        viewport: &ViewportResource,
    ) -> Option<ClosestShapeIndexDistance> {
        let mut closest_shape_index = None;
        let mut closest_distance_sq = std::f32::MAX;

        // Linearly iterate the shapes to find the closest one to the mouse position
        for i in 0..self.shapes.len() {
            let shape = &self.shapes[i];

            let distance_sq = match &shape.shape {
                Shape::Line(line) => distance_to_segment_sq(
                    test_position,
                    viewport.world_space_to_ui_space(line.p0),
                    viewport.world_space_to_ui_space(line.p1),
                ),
                Shape::CircleOutline(circle) => {
                    // This is an odd kludge, but we want to work in ui space. However, the radius in ui space won't match the radius in
                    // world space.
                    let position_on_outline = circle.center + glam::vec2(circle.radius, 0.0);
                    let scaled_center = viewport.world_space_to_ui_space(circle.center);
                    let scaled_position_on_outline =
                        viewport.world_space_to_ui_space(position_on_outline);
                    let scaled_radius =
                        f32::abs(scaled_position_on_outline.x() - scaled_center.x());

                    distance_to_circle_outline_sq(test_position, scaled_center, scaled_radius)
                }
            };

            if distance_sq < closest_distance_sq {
                closest_distance_sq = distance_sq;
                closest_shape_index = Some(i);
            }
        }

        Some(ClosestShapeIndexDistance {
            index: closest_shape_index?,
            distance_sq: closest_distance_sq,
        })
    }

    pub fn update(
        &mut self,
        input_state: &InputState,
        viewport: &ViewportResource,
    ) {
        // See if the user interacted with anything. If they did, then cache it. User code would need to
        // check this and possibly check against other clickable things (like if an object in the editor was clicked.)
        // We likely need a measure of depth if we draw in the 3D world in a way that can be occluded.

        // Get mouse UI-space position
        let mouse_position = input_state.mouse_position();
        let closest_shape = self.get_closest_shape(mouse_position, viewport);

        if let Some(closest_shape) = closest_shape {
            self.closest_shape_to_mouse.id = self.shapes[closest_shape.index].id.clone();
            self.closest_shape_to_mouse.distance_sq = closest_shape.distance_sq;
        } else {
            self.closest_shape_to_mouse.id.clear();
            self.closest_shape_to_mouse.distance_sq = std::f32::MAX;
        }

        self.shape_last_interacted.clear();

        // Check for clicking/dragging for each mouse button
        for mouse_button_index in 0..InputState::MOUSE_BUTTON_COUNT {
            let mouse_button_index = mouse_button_index as usize;
            let mouse_button = InputState::mouse_index_to_button(mouse_button_index).unwrap();

            // See if mouse button went down and is over a shape. Check this first because we may need to know this in the next block
            // that handles click/drag detection.
            if let Some(down_position) = input_state.mouse_button_went_down_position(mouse_button) {
                let down_position = down_position;
                if let Some(closest_shape) = self.get_closest_shape(down_position, viewport) {
                    if closest_shape.distance_sq < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE_SQ {
                        self.mouse_is_down_on_shape[mouse_button_index] =
                            Some(EditorShapeClickedState {
                                click_position: down_position,
                                shape_id: self.shapes[closest_shape.index].id.clone(),
                            });
                    }
                }
            }

            self.shape_just_clicked[mouse_button_index] = None;
            self.shape_drag_just_finished[mouse_button_index] = None;
            // Don't clear shape_drag_in_progress here, we need it in the next check

            if let Some(current_drag_in_progress) = &self.shape_drag_in_progress[mouse_button_index]
            {
                // Check several cases (drag in progress, drag finished, unexpectedly not dragging) and set
                // shape_drag_in_progress, shape_drag_just_finished. May also set shape_last_interacted.
                if let Some(input_state_drag_in_progress) =
                    &input_state.mouse_drag_in_progress(mouse_button)
                {
                    // update shape drag state
                    self.shape_last_interacted = current_drag_in_progress.shape_id.clone();

                    let world_space_end_position =
                        viewport.ui_space_to_world_space(input_state_drag_in_progress.end_position);
                    let delta = world_space_end_position
                        - (current_drag_in_progress.world_space_begin_position
                            + current_drag_in_progress.world_space_accumulated_frame_delta);

                    self.shape_drag_in_progress[mouse_button_index] = Some(EditorShapeDragState {
                        begin_position: input_state_drag_in_progress.begin_position,
                        end_position: input_state_drag_in_progress.end_position,
                        previous_frame_delta: input_state_drag_in_progress.previous_frame_delta,
                        accumulated_frame_delta: input_state_drag_in_progress
                            .accumulated_frame_delta,
                        world_space_begin_position: current_drag_in_progress
                            .world_space_begin_position,
                        world_space_end_position,
                        world_space_previous_frame_delta: delta,
                        world_space_accumulated_frame_delta: delta
                            + current_drag_in_progress.world_space_accumulated_frame_delta,
                        shape_id: current_drag_in_progress.shape_id.clone(),
                    });
                    self.shape_drag_just_finished[mouse_button_index] = None;
                } else if let Some(input_state_drag_just_finished) =
                    &input_state.mouse_drag_just_finished(mouse_button)
                {
                    // update mouse drag

                    let world_space_end_position = viewport
                        .ui_space_to_world_space(input_state_drag_just_finished.end_position);
                    let delta = world_space_end_position
                        - (current_drag_in_progress.world_space_begin_position
                            + current_drag_in_progress.world_space_accumulated_frame_delta);

                    self.shape_last_interacted = current_drag_in_progress.shape_id.clone();
                    self.shape_drag_just_finished[mouse_button_index] =
                        Some(EditorShapeDragState {
                            begin_position: input_state_drag_just_finished.begin_position,
                            end_position: input_state_drag_just_finished.end_position,
                            previous_frame_delta: input_state_drag_just_finished
                                .previous_frame_delta,
                            accumulated_frame_delta: input_state_drag_just_finished
                                .accumulated_frame_delta,
                            world_space_begin_position: current_drag_in_progress
                                .world_space_begin_position,
                            world_space_end_position,
                            world_space_previous_frame_delta: delta,
                            world_space_accumulated_frame_delta: delta
                                + current_drag_in_progress.world_space_accumulated_frame_delta,
                            shape_id: current_drag_in_progress.shape_id.clone(),
                        });
                    self.shape_drag_in_progress[mouse_button_index] = None;
                } else {
                    // This is unexpected but we should gracefully recover here. We expect that if shape
                    // dragging is in progress, that mouse dragging is in progress or just finished
                    log::warn!("Unexpected input_state when updating editor shapes");
                    self.shape_drag_in_progress[mouse_button_index] = None;
                }
            } else {
                // Shape drag is not in progress
                self.shape_drag_in_progress[mouse_button_index] = None;

                // Can't click or drag a shape unless it's nearby
                if self.closest_shape_to_mouse.distance_sq
                    < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE_SQ
                {
                    //need to use the click position isntead of mosue position
                    if let Some(mouse_drag_in_progress) =
                        input_state.mouse_drag_in_progress(mouse_button)
                    {
                        // we started dragging a shape
                        if let Some(down_on_shape) =
                            &self.mouse_is_down_on_shape[mouse_button_index]
                        {
                            if let Some(closest_shape) = self
                                .get_closest_shape(mouse_drag_in_progress.begin_position, viewport)
                            {
                                let shape = &self.shapes[closest_shape.index];
                                if closest_shape.distance_sq
                                    < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE_SQ
                                    && down_on_shape.shape_id == shape.id
                                {
                                    let world_space_begin_position = viewport
                                        .ui_space_to_world_space(
                                            mouse_drag_in_progress.begin_position,
                                        );
                                    let world_space_end_position = viewport
                                        .ui_space_to_world_space(
                                            mouse_drag_in_progress.end_position,
                                        );
                                    let world_space_previous_frame_delta =
                                        world_space_end_position - world_space_begin_position;

                                    self.shape_last_interacted =
                                        self.closest_shape_to_mouse.id.clone();
                                    self.shape_drag_in_progress[mouse_button_index] =
                                        Some(EditorShapeDragState {
                                            begin_position: mouse_drag_in_progress.begin_position,
                                            end_position: mouse_drag_in_progress.end_position,
                                            previous_frame_delta: mouse_drag_in_progress
                                                .previous_frame_delta,
                                            accumulated_frame_delta: mouse_drag_in_progress
                                                .accumulated_frame_delta,
                                            world_space_begin_position,
                                            world_space_end_position,
                                            world_space_previous_frame_delta,
                                            world_space_accumulated_frame_delta:
                                                world_space_previous_frame_delta,
                                            shape_id: self.closest_shape_to_mouse.id.clone(),
                                        });
                                }
                            }
                        }
                    } else if let Some(just_clicked_position) =
                        input_state.mouse_button_just_clicked_position(mouse_button)
                    {
                        let just_clicked_position = just_clicked_position;
                        // check if we clicked a shape
                        if let Some(down_on_shape) =
                            &self.mouse_is_down_on_shape[mouse_button_index]
                        {
                            if let Some(closest_shape) =
                                self.get_closest_shape(just_clicked_position, viewport)
                            {
                                let shape = &self.shapes[closest_shape.index];
                                if closest_shape.distance_sq
                                    < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE_SQ
                                    && down_on_shape.shape_id == shape.id
                                {
                                    self.shape_last_interacted = shape.id.clone();
                                    self.shape_just_clicked[mouse_button_index] =
                                        Some(EditorShapeClickedState {
                                            click_position: just_clicked_position,
                                            shape_id: shape.id.clone(),
                                        });
                                }
                            }
                        }
                    }
                }
            }

            // Handle mouse up = mouse is no longer down on a shape
            if input_state
                .mouse_button_went_up_position(mouse_button)
                .is_some()
            {
                self.mouse_is_down_on_shape[mouse_button_index] = None;
            }
        }

        self.shapes.clear();
    }

    pub fn is_interacting_with_anything(&self) -> bool {
        !self.shape_last_interacted.is_empty()
    }

    pub fn is_shape_just_clicked(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        self.shape_just_clicked[InputState::mouse_button_to_index(mouse_button).unwrap()].is_some()
    }

    pub fn shape_just_clicked(
        &self,
        mouse_button: MouseButton,
    ) -> &Option<EditorShapeClickedState> {
        &self.shape_just_clicked[InputState::mouse_button_to_index(mouse_button).unwrap()]
    }

    pub fn is_shape_drag_in_progress(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        self.shape_drag_in_progress[InputState::mouse_button_to_index(mouse_button).unwrap()]
            .is_some()
    }

    pub fn shape_drag_in_progress(
        &self,
        mouse_button: MouseButton,
    ) -> &Option<EditorShapeDragState> {
        &self.shape_drag_in_progress[InputState::mouse_button_to_index(mouse_button).unwrap()]
    }

    pub fn is_shape_drag_just_finished(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        self.shape_drag_just_finished[InputState::mouse_button_to_index(mouse_button).unwrap()]
            .is_some()
    }

    pub fn shape_drag_just_finished(
        &self,
        mouse_button: MouseButton,
    ) -> &Option<EditorShapeDragState> {
        &self.shape_drag_just_finished[InputState::mouse_button_to_index(mouse_button).unwrap()]
    }

    pub fn is_shape_drag_in_progress_or_just_finished(
        &self,
        mouse_button: MouseButton,
    ) -> bool {
        self.is_shape_drag_in_progress(mouse_button)
            || self.is_shape_drag_just_finished(mouse_button)
    }

    pub fn shape_drag_in_progress_or_just_finished(
        &self,
        mouse_button: MouseButton,
    ) -> &Option<EditorShapeDragState> {
        if self.is_shape_drag_just_finished(mouse_button) {
            return self.shape_drag_just_finished(mouse_button);
        }

        self.shape_drag_in_progress(mouse_button)
    }
}
