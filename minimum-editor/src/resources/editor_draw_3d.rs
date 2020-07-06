use minimum_game::resources::DebugDraw3DResource;
use minimum_game::resources::DebugDraw3DDepthBehavior;
use minimum_game::resources::ViewportResource;
use minimum_math::NormalizedRay;

use minimum_game::input::InputState;
use minimum_game::input::MouseButton;

struct Line {
    p0: glam::Vec3,
    p1: glam::Vec3,
}

struct Sphere {
    center: glam::Vec3,
    radius: f32,
}

enum Shape {
    Line(Line),
    Sphere(Sphere),
}

struct ShapeWithId {
    id: String,
    shape: Shape,
}

impl ShapeWithId {
    fn new_line(
        id: String,
        p0: glam::Vec3,
        p1: glam::Vec3,
    ) -> Self {
        ShapeWithId {
            id,
            shape: Shape::Line(Line { p0, p1 }),
        }
    }

    fn new_sphere(
        id: String,
        center: glam::Vec3,
        radius: f32,
    ) -> Self {
        ShapeWithId {
            id,
            shape: Shape::Sphere(Sphere { center, radius }),
        }
    }
}

struct ClosestShapeIdDistance {
    id: String,
    distance: f32,
}

struct ClosestShapeIndexDistance {
    index: usize,
    distance: f32,
}

#[derive(Clone, Debug)]
struct Ray {
    origin: glam::Vec3,
    direction: glam::Vec3
}

#[derive(Clone, Debug)]
pub struct EditorDraw3DShapeClickedState {
    pub click_position: glam::Vec2, // screen space
    //pub click_ray: Ray,
    pub shape_id: String,
}

#[derive(Clone, Debug)]
pub struct EditorDraw3DShapeDragState {
    pub begin_position: glam::Vec2, // screen space
    pub end_position: glam::Vec2, // screen space
    pub previous_frame_delta: glam::Vec2, // screen space
    pub accumulated_frame_delta: glam::Vec2, // screen space
    pub world_space_begin_position: glam::Vec3,
    pub world_space_end_position: glam::Vec3,
    pub world_space_previous_frame_delta: glam::Vec3,
    pub world_space_accumulated_frame_delta: glam::Vec3,
    pub shape_id: String,
}

const MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE: f32 = 30.0;

//TODO: Rename to EditorShapeDrawResource.. or maybe gizmo?
//TODO: How does this interact with the select/inspect registry?
pub struct EditorDraw3DResource {
    shapes: Vec<ShapeWithId>,
    shape_just_clicked: [Option<EditorDraw3DShapeClickedState>; InputState::MOUSE_BUTTON_COUNT as usize],
    shape_drag_in_progress: [Option<EditorDraw3DShapeDragState>; InputState::MOUSE_BUTTON_COUNT as usize],
    shape_drag_just_finished:
        [Option<EditorDraw3DShapeDragState>; InputState::MOUSE_BUTTON_COUNT as usize],
    mouse_is_down_on_shape:
        [Option<EditorDraw3DShapeClickedState>; InputState::MOUSE_BUTTON_COUNT as usize],
    shape_last_interacted: String,
    closest_shape_to_mouse: ClosestShapeIdDistance,
}

impl EditorDraw3DResource {
    pub fn new() -> Self {
        EditorDraw3DResource {
            shapes: vec![],
            shape_just_clicked: Default::default(),
            shape_drag_in_progress: Default::default(),
            shape_drag_just_finished: Default::default(),
            mouse_is_down_on_shape: Default::default(),
            shape_last_interacted: "".to_string(),
            closest_shape_to_mouse: ClosestShapeIdDistance {
                id: "".to_string(),
                distance: std::f32::MAX,
            },
        }
    }

    // Input here is world space
    pub fn add_line(
        &mut self,
        id: &str,
        debug_draw: &mut DebugDraw3DResource,
        p0: glam::Vec3,
        p1: glam::Vec3,
        mut color: glam::Vec4,
        depth_behavior: DebugDraw3DDepthBehavior,
    ) {
        if self.closest_shape_to_mouse.id == id
            && self.closest_shape_to_mouse.distance < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE
        {
            color = glam::vec4(1.0, 0.0, 0.0, 1.0);
        }

        debug_draw.add_line(p0, p1, color, depth_behavior);
        self.shapes
            .push(ShapeWithId::new_line(id.to_string(), p0, p1));
    }

    // Input here is world space
    // pub fn add_sphere(
    //     &mut self,
    //     id: &str,
    //     debug_draw: &mut DebugDraw3DResource,
    //     center: glam::Vec3,
    //     radius: f32,
    //     segments: u32,
    //     mut color: glam::Vec4,
    //     depth_behavior: DebugDraw3DDepthBehavior,
    // ) {
    //     if self.closest_shape_to_mouse.id == id
    //         && self.closest_shape_to_mouse.distance < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE
    //     {
    //         color = glam::vec4(1.0, 0.0, 0.0, 1.0);
    //     }
    //
    //     debug_draw.add_sphere(center, radius, color, depth_behavior, segments);
    //     self.shapes.push(ShapeWithId::new_sphere(
    //         id.to_string(),
    //         center,
    //         radius,
    //     ));
    // }

    // test_position is ui space, the shapes we keep are in world space
    //TODO: Consider converting to ui space immediately
    fn get_closest_shape(
        &self,
        ray: NormalizedRay,
    ) -> Option<ClosestShapeIndexDistance> {

        //println!("get_closest_shape {:?}", ray);

        let mut closest_shape_index = None;
        let mut closest_t = 1.0;

        // Linearly iterate the shapes to find the closest one to the mouse position
        for i in 0..self.shapes.len() {
            let shape = &self.shapes[i];

            let t = match &shape.shape {
                Shape::Line(line) => {

                    let mut line_dir = line.p1 - line.p0;
                    let line_length = line_dir.length();
                    line_dir = line_dir / line_length;

                    let result = minimum_math::functions::ray_intersect(
                        ray,
                        NormalizedRay {
                            origin: line.p0,
                            dir: line_dir,
                            length: line_length
                        }
                    );

                    result.t0
                },
                Shape::Sphere(circle) => {
                    // // This is an odd kludge, but we want to work in ui space. However, the radius in ui space won't match the radius in
                    // // world space.
                    // let position_on_outline = circle.center + glam::vec2(circle.radius, 0.0);
                    // let scaled_center = viewport.world_space_to_ui_space(circle.center);
                    // let scaled_position_on_outline =
                    //     viewport.world_space_to_ui_space(position_on_outline);
                    // let scaled_radius =
                    //     f32::abs(scaled_position_on_outline.x() - scaled_center.x());
                    //
                    // minimum_math::functions::distance_to_sphere_sq(test_position, scaled_center, scaled_radius)

                    2.0
                }
            };

            if t < closest_t || closest_shape_index.is_none() {
                closest_t = t;
                closest_shape_index = Some(i);
            }
        }

        if closest_t > 1.0 {
            closest_shape_index = None;
        }

        let closest_distance = ray.length * closest_t;

        Some(ClosestShapeIndexDistance {
            index: closest_shape_index?,
            distance: closest_distance,
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
        let mouse_ray = viewport.viewport_space_to_ray(mouse_position);
        let closest_shape = self.get_closest_shape(mouse_ray);

        if let Some(closest_shape) = closest_shape {
            self.closest_shape_to_mouse.id = self.shapes[closest_shape.index].id.clone();
            self.closest_shape_to_mouse.distance = closest_shape.distance;
        } else {
            self.closest_shape_to_mouse.id.clear();
            self.closest_shape_to_mouse.distance = std::f32::MAX;
        }

        self.shape_last_interacted.clear();

        // Check for clicking/dragging for each mouse button
        for mouse_button_index in 0..InputState::MOUSE_BUTTON_COUNT {
            let mouse_button_index = mouse_button_index as usize;
            let mouse_button = InputState::mouse_index_to_button(mouse_button_index).unwrap();

            // See if mouse button went down and is over a shape. Check this first because we may need to know this in the next block
            // that handles click/drag detection.
            if let Some(down_position) = input_state.mouse_button_went_down_position(mouse_button) {
                let down_ray = viewport.viewport_space_to_ray(down_position);
                if let Some(closest_shape) = self.get_closest_shape(down_ray) {
                    if closest_shape.distance < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE {
                        self.mouse_is_down_on_shape[mouse_button_index] =
                            Some(EditorDraw3DShapeClickedState {
                                click_position: down_position,
                                //click_ray:
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
                        viewport.viewport_space_to_world_space(input_state_drag_in_progress.end_position, 0.0);
                    let delta = world_space_end_position
                        - (current_drag_in_progress.world_space_begin_position
                            + current_drag_in_progress.world_space_accumulated_frame_delta);

                    self.shape_drag_in_progress[mouse_button_index] = Some(EditorDraw3DShapeDragState {
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
                        .viewport_space_to_world_space(input_state_drag_just_finished.end_position, 0.0);
                    let delta = world_space_end_position
                        - (current_drag_in_progress.world_space_begin_position
                            + current_drag_in_progress.world_space_accumulated_frame_delta);

                    self.shape_last_interacted = current_drag_in_progress.shape_id.clone();
                    self.shape_drag_just_finished[mouse_button_index] =
                        Some(EditorDraw3DShapeDragState {
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
                if self.closest_shape_to_mouse.distance
                    < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE
                {
                    //need to use the click position isntead of mosue position
                    if let Some(mouse_drag_in_progress) =
                        input_state.mouse_drag_in_progress(mouse_button)
                    {
                        // we started dragging a shape
                        if let Some(down_on_shape) =
                            &self.mouse_is_down_on_shape[mouse_button_index]
                        {
                            let begin_ray = viewport.viewport_space_to_ray(mouse_drag_in_progress.begin_position);
                            if let Some(closest_shape) = self
                                .get_closest_shape(begin_ray)
                            {
                                let shape = &self.shapes[closest_shape.index];
                                if closest_shape.distance
                                    < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE
                                    && down_on_shape.shape_id == shape.id
                                {
                                    let world_space_begin_position = viewport
                                        .viewport_space_to_world_space(
                                            mouse_drag_in_progress.begin_position,
                                            0.0
                                        );
                                    let world_space_end_position = viewport
                                        .viewport_space_to_world_space(
                                            mouse_drag_in_progress.end_position,
                                            0.0
                                        );
                                    let world_space_previous_frame_delta =
                                        world_space_end_position - world_space_begin_position;

                                    self.shape_last_interacted =
                                        self.closest_shape_to_mouse.id.clone();
                                    self.shape_drag_in_progress[mouse_button_index] =
                                        Some(EditorDraw3DShapeDragState {
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
                        // check if we clicked a shape
                        if let Some(down_on_shape) =
                            &self.mouse_is_down_on_shape[mouse_button_index]
                        {
                            let just_clicked_ray = viewport.viewport_space_to_ray(just_clicked_position);
                            if let Some(closest_shape) =
                                self.get_closest_shape(just_clicked_ray)
                            {
                                let shape = &self.shapes[closest_shape.index];
                                if closest_shape.distance
                                    < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE
                                    && down_on_shape.shape_id == shape.id
                                {
                                    self.shape_last_interacted = shape.id.clone();
                                    self.shape_just_clicked[mouse_button_index] =
                                        Some(EditorDraw3DShapeClickedState {
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
    ) -> &Option<EditorDraw3DShapeClickedState> {
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
    ) -> &Option<EditorDraw3DShapeDragState> {
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
    ) -> &Option<EditorDraw3DShapeDragState> {
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
    ) -> &Option<EditorDraw3DShapeDragState> {
        if self.is_shape_drag_just_finished(mouse_button) {
            return self.shape_drag_just_finished(mouse_button);
        }

        self.shape_drag_in_progress(mouse_button)
    }
}
