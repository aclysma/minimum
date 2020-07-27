use minimum_game::resources::DebugDraw3DResource;
use minimum_game::resources::DebugDraw3DDepthBehavior;
use minimum_game::resources::DebugDraw2DResource;
use minimum_game::resources::ViewportResource;
use minimum_math::NormalizedRay;

use minimum_game::input::InputState;
use minimum_game::input::MouseButton;

#[derive(Debug, Copy, Clone)]
pub struct PlaneConstraint {
    pub basis: glam::Vec3,
    pub normal: glam::Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct LineConstraint {
    pub basis: glam::Vec3,
    pub dir: glam::Vec3
}

#[derive(Debug, Copy, Clone)]
pub enum EditorDraw3DConstraint {
    Plane(PlaneConstraint),
    Line(LineConstraint)
}

impl EditorDraw3DConstraint {
    pub fn x_line(basis: glam::Vec3) -> Self {
        EditorDraw3DConstraint::Line(LineConstraint {
            basis,
            dir: glam::Vec3::unit_x()
        })
    }

    pub fn y_line(basis: glam::Vec3) -> Self {
        EditorDraw3DConstraint::Line(LineConstraint {
            basis,
            dir: glam::Vec3::unit_y()
        })
    }

    pub fn z_line(basis: glam::Vec3) -> Self {
        EditorDraw3DConstraint::Line(LineConstraint {
            basis,
            dir: glam::Vec3::unit_z()
        })
    }

    pub fn xy_plane(basis: glam::Vec3) -> Self {
        EditorDraw3DConstraint::Plane(PlaneConstraint {
            basis,
            normal: glam::Vec3::unit_z()
        })
    }

    pub fn xz_plane(basis: glam::Vec3) -> Self {
        EditorDraw3DConstraint::Plane(PlaneConstraint {
            basis,
            normal: glam::Vec3::unit_y()
        })
    }

    pub fn yz_plane(basis: glam::Vec3) -> Self {
        EditorDraw3DConstraint::Plane(PlaneConstraint {
            basis,
            normal: glam::Vec3::unit_x()
        })
    }

    pub fn basis(&self) -> glam::Vec3 {
        match self {
            EditorDraw3DConstraint::Line(constraint) => {
                constraint.basis
            },
            EditorDraw3DConstraint::Plane(constraint) => {
                constraint.basis
            }
        }
    }

    pub fn project_mouse_to_world(&self, mouse_ray: NormalizedRay) -> glam::Vec3 {
        match self {
            EditorDraw3DConstraint::Line(constraint) => {
                let line_ray = NormalizedRay {
                    origin: constraint.basis,
                    dir: constraint.dir,
                    length: 1.0
                };

                let result = minimum_math::functions::line_line_intersect_3d(
                    mouse_ray,
                    line_ray,
                );

                // Closest point on the line
                result.c1
            },
            EditorDraw3DConstraint::Plane(constraint) => {
                let intersection = minimum_math::functions::line_plane_intersect_3d(
                    mouse_ray.origin,
                    mouse_ray.dir,
                    constraint.basis,
                    constraint.normal
                );

                match intersection {
                    minimum_math::functions::PlaneIntersectResult::NoIntersection => constraint.basis,
                    minimum_math::functions::PlaneIntersectResult::LineContainedWithinPlane => constraint.basis,
                    minimum_math::functions::PlaneIntersectResult::Intersect(position) => position
                }
            }
        }
    }
}

struct Line {
    p0: glam::Vec3,
    p1: glam::Vec3,
    constraint: EditorDraw3DConstraint
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
        constraint: EditorDraw3DConstraint
    ) -> Self {
        ShapeWithId {
            id,
            shape: Shape::Line(Line { p0, p1, constraint }),
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

#[derive(Debug)]
struct ClosestShapeIndexDistance {
    index: usize,
    constraint: EditorDraw3DConstraint,
    distance: f32,
}

#[derive(Clone, Debug)]
struct Ray {
    origin: glam::Vec3,
    direction: glam::Vec3
}

#[derive(Clone, Debug)]
pub struct EditorDraw3DShapeClickedState {
    pub mouse_position: glam::Vec2, // screen space of mouse
    pub shape_id: String,
}

#[derive(Clone, Debug)]
pub struct EditorDraw3DShapeDragState {
    pub shape_id: String,
    pub constraint: EditorDraw3DConstraint,
    pub mouse_begin_position: glam::Vec2,
    pub mouse_end_position: glam::Vec2,
    pub mouse_previous_frame_delta: glam::Vec2,
    pub mouse_accumulated_frame_delta: glam::Vec2,
    pub world_space_begin_position: glam::Vec3,
    pub world_space_end_position: glam::Vec3,
    pub world_space_previous_frame_delta: glam::Vec3,
    pub world_space_accumulated_frame_delta: glam::Vec3,
}

const MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE: f32 = 10.0;

//TODO: Rename to EditorShapeDrawResource.. or maybe gizmo?
//TODO: How does this interact with the select/inspect registry?
pub struct EditorDraw3DResource {
    shapes: Vec<ShapeWithId>,
    shape_just_clicked: [Option<EditorDraw3DShapeClickedState>; InputState::MOUSE_BUTTON_COUNT as usize],
    shape_drag_in_progress: [Option<EditorDraw3DShapeDragState>; InputState::MOUSE_BUTTON_COUNT as usize],
    shape_drag_just_finished:
        [Option<EditorDraw3DShapeDragState>; InputState::MOUSE_BUTTON_COUNT as usize],
    mouse_previous_down_on_shape:
        [Option<EditorDraw3DShapeClickedState>; InputState::MOUSE_BUTTON_COUNT as usize],
    shape_last_interacted: String,
    closest_shape_to_mouse: ClosestShapeIdDistance,
}

#[derive(Debug, PartialEq)]
enum GetClosestShapeCallReason {
    Hover,
    Down,
    DragInProgress,
    JustClicked
}

impl EditorDraw3DResource {
    pub fn new() -> Self {
        EditorDraw3DResource {
            shapes: vec![],
            shape_just_clicked: Default::default(),
            shape_drag_in_progress: Default::default(),
            shape_drag_just_finished: Default::default(),
            mouse_previous_down_on_shape: Default::default(),
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
        constraint: EditorDraw3DConstraint,
        //basis: glam::Vec3,
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
            .push(ShapeWithId::new_line(
                id.to_string(),
                p0,
                p1,
                constraint
            )
        );
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

    fn get_closest_shape(
        &self,
        mouse_position: glam::Vec2,
        viewport: &ViewportResource,
        debug_draw_3d: &mut DebugDraw3DResource,
        debug_draw_2d: &mut DebugDraw2DResource,
        call_reason: GetClosestShapeCallReason
    ) -> Option<ClosestShapeIndexDistance> {
        struct ShapeIntersectResult {
            shape_index: usize,
            t: f32,
            constraint: EditorDraw3DConstraint,
            distance_2d: f32
        }

        let mut closest_intersect : Option<ShapeIntersectResult> = None;

        let ray = viewport.viewport_space_to_ray(mouse_position);

        // Linearly iterate the shapes to find the closest one to the mouse position
        for i in 0..self.shapes.len() {
            let shape = &self.shapes[i];

            let result = match &shape.shape {
                Shape::Line(line) => {
                    // Figure out where the 3d line is in world space
                    let mut line_dir = (line.p1 - line.p0).normalize();
                    let line_length = line_dir.length();
                    line_dir = line_dir / line_length;

                    // Project the 3d line to 2d
                    let line_p0_2d = viewport.world_space_to_viewport_space(line.p0);
                    let line_p1_2d = viewport.world_space_to_viewport_space(line.p1);

                    // Do 2D intersection to find closest point on the line in screen space
                    let result_2d = minimum_math::functions::point_segment_intersect_2d(mouse_position, line_p0_2d, line_p1_2d);
                    let closest_point_2d = result_2d.closest_point;

                    // Now produce a ray at that 2d coordinate to find the intersection in 3d space and viewport depth
                    let closest_point_ray = viewport.viewport_space_to_ray(closest_point_2d);
                    let result_3d = minimum_math::functions::ray_ray_intersect_3d(
                        closest_point_ray,
                        NormalizedRay {
                            origin: line.p0,
                            dir: line_dir,
                            length: line_length
                        },
                        1.0,
                        1.0
                    );

                    //TODO: Adjust by screen DPI?
                    let distance_2d = result_2d.distance_sq.sqrt();
                    if distance_2d < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE {
                        Some(ShapeIntersectResult {
                            shape_index: i,
                            t: result_3d.t0,
                            constraint: line.constraint,
                            distance_2d
                        })
                    } else {
                        None
                    }
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

                    //std::f32::MAX
                    None
                }
            };

            if let Some(result) = result {
                if closest_intersect.is_none() || result.t < closest_intersect.as_ref().unwrap().t {
                    closest_intersect = Some(result);
                }
            }
        }

        if let Some(closest_intersect) = closest_intersect {
            if closest_intersect.t <= ray.length {
                Some(ClosestShapeIndexDistance {
                    index: closest_intersect.shape_index,
                    constraint: closest_intersect.constraint,
                    distance: closest_intersect.distance_2d,
                })
            } else {
                None
            }
        } else {
            None
        }

    }

    pub fn update(
        &mut self,
        input_state: &InputState,
        viewport: &ViewportResource,
        debug_draw_3d: &mut DebugDraw3DResource,
        debug_draw_2d: &mut DebugDraw2DResource,
    ) {
        // See if the user interacted with anything. If they did, then cache it. User code would need to
        // check this and possibly check against other clickable things (like if an object in the editor was clicked.)
        // We likely need a measure of depth if we draw in the 3D world in a way that can be occluded.

        // Get mouse UI-space position
        let mouse_position = input_state.mouse_position();
        let closest_shape = self.get_closest_shape(mouse_position, viewport, debug_draw_3d, debug_draw_2d, GetClosestShapeCallReason::Hover);

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
            if let Some(mouse_previous_down_position) = input_state.mouse_button_went_down_position(mouse_button) {
                if let Some(closest_shape) = self.get_closest_shape(mouse_previous_down_position, viewport, debug_draw_3d, debug_draw_2d, GetClosestShapeCallReason::Down) {
                    if closest_shape.distance < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE {
                        self.mouse_previous_down_on_shape[mouse_button_index] =
                            Some(EditorDraw3DShapeClickedState {
                                mouse_position: mouse_previous_down_position,
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
                //
                // Shape was already being dragged
                //
                // Check several cases (drag in progress, drag finished, unexpectedly not dragging) and set
                // shape_drag_in_progress, shape_drag_just_finished. May also set shape_last_interacted.
                //
                if let Some(input_state_drag_in_progress) =
                    &input_state.mouse_drag_in_progress(mouse_button)
                {
                    let end_position_mouse_ray = viewport.viewport_space_to_ray(input_state_drag_in_progress.end_position);
                    let world_space_end_position = current_drag_in_progress.constraint.project_mouse_to_world(end_position_mouse_ray);
                    let delta = world_space_end_position
                        - (current_drag_in_progress.world_space_begin_position
                        + current_drag_in_progress.world_space_accumulated_frame_delta);

                    self.shape_last_interacted = current_drag_in_progress.shape_id.clone();
                    self.shape_drag_in_progress[mouse_button_index] = Some(EditorDraw3DShapeDragState {
                        mouse_begin_position: input_state_drag_in_progress.begin_position,
                        mouse_end_position: input_state_drag_in_progress.end_position,
                        mouse_previous_frame_delta: input_state_drag_in_progress.previous_frame_delta,
                        mouse_accumulated_frame_delta: input_state_drag_in_progress
                            .accumulated_frame_delta,
                        world_space_begin_position: current_drag_in_progress
                            .world_space_begin_position,
                        world_space_end_position,
                        world_space_previous_frame_delta: delta,
                        world_space_accumulated_frame_delta: delta
                            + current_drag_in_progress.world_space_accumulated_frame_delta,
                        shape_id: current_drag_in_progress.shape_id.clone(),
                        constraint: current_drag_in_progress.constraint,
                    });
                    self.shape_drag_just_finished[mouse_button_index] = None;
                } else if let Some(input_state_drag_just_finished) =
                    &input_state.mouse_drag_just_finished(mouse_button)
                {
                    println!("drag finish");
                    //
                    // Drag finished, finalize position changes
                    //
                    let end_position_mouse_ray = viewport.viewport_space_to_ray(input_state_drag_just_finished.end_position);
                    let world_space_end_position = current_drag_in_progress.constraint.project_mouse_to_world(end_position_mouse_ray);
                    let delta = world_space_end_position
                        - (current_drag_in_progress.world_space_begin_position
                            + current_drag_in_progress.world_space_accumulated_frame_delta);

                    self.shape_last_interacted = current_drag_in_progress.shape_id.clone();
                    self.shape_drag_just_finished[mouse_button_index] =
                        Some(EditorDraw3DShapeDragState {
                            mouse_begin_position: input_state_drag_just_finished.begin_position,
                            mouse_end_position: input_state_drag_just_finished.end_position,
                            mouse_previous_frame_delta: input_state_drag_just_finished
                                .previous_frame_delta,
                            mouse_accumulated_frame_delta: input_state_drag_just_finished
                                .accumulated_frame_delta,
                            world_space_begin_position: current_drag_in_progress
                                .world_space_begin_position,
                            world_space_end_position,
                            world_space_previous_frame_delta: delta,
                            world_space_accumulated_frame_delta: delta
                                + current_drag_in_progress.world_space_accumulated_frame_delta,
                            shape_id: current_drag_in_progress.shape_id.clone(),

                            constraint: current_drag_in_progress.constraint
                        });
                    self.shape_drag_in_progress[mouse_button_index] = None;
                } else {
                    println!("drag end abruptly");
                    //
                    // Shape drag is in progress but mouse drag stopped without a finished event.
                    //
                    // This is unexpected but we should gracefully recover here. We expect that if shape
                    // dragging is in progress, that mouse dragging is in progress or just finished
                    //
                    log::warn!("Unexpected input_state when updating editor shapes");
                    self.shape_drag_in_progress[mouse_button_index] = None;
                }
            } else {
                //
                // Shape drag is not in progress
                //
                //self.shape_drag_in_progress[mouse_button_index] = None;

                // Can't click or drag a shape unless it's nearby
                if self.closest_shape_to_mouse.distance
                    < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE
                {
                    //need to use the click position isntead of mosue position
                    if let Some(mouse_drag_in_progress) =
                        input_state.mouse_drag_in_progress(mouse_button)
                    {
                        //
                        // A mouse drag is in progress but the shape drag did not start yet
                        //

                        // we started dragging a shape
                        if let Some(down_on_shape) =
                            &self.mouse_previous_down_on_shape[mouse_button_index]
                        {
                            if let Some(closest_shape) = self
                                .get_closest_shape(mouse_drag_in_progress.begin_position, viewport, debug_draw_3d, debug_draw_2d, GetClosestShapeCallReason::DragInProgress)
                            {
                                let shape = &self.shapes[closest_shape.index];
                                if closest_shape.distance
                                    < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE
                                    && down_on_shape.shape_id == shape.id
                                {
                                    let begin_position_mouse_ray = viewport.viewport_space_to_ray(mouse_drag_in_progress.begin_position);
                                    let world_space_begin_position = closest_shape.constraint.project_mouse_to_world(begin_position_mouse_ray);
                                    let end_position_mouse_ray = viewport.viewport_space_to_ray(mouse_drag_in_progress.end_position);
                                    let world_space_end_position = closest_shape.constraint.project_mouse_to_world(end_position_mouse_ray);
                                    let world_space_previous_frame_delta =
                                        world_space_end_position - world_space_begin_position;

                                    self.shape_last_interacted =
                                        self.closest_shape_to_mouse.id.clone();
                                    self.shape_drag_in_progress[mouse_button_index] =
                                        Some(EditorDraw3DShapeDragState {
                                            mouse_begin_position: mouse_drag_in_progress.begin_position,
                                            mouse_end_position: mouse_drag_in_progress.end_position,
                                            mouse_previous_frame_delta: mouse_drag_in_progress
                                                .previous_frame_delta,
                                            mouse_accumulated_frame_delta: mouse_drag_in_progress
                                                .accumulated_frame_delta,
                                            world_space_begin_position,
                                            world_space_end_position,
                                            world_space_previous_frame_delta,
                                            world_space_accumulated_frame_delta:
                                                world_space_previous_frame_delta,
                                            shape_id: self.closest_shape_to_mouse.id.clone(),

                                            constraint: closest_shape.constraint
                                        });
                                }
                            }
                        }
                    } else if let Some(just_clicked_mouse_position) =
                        input_state.mouse_button_just_clicked_position(mouse_button)
                    {
                        //
                        // A shape was clicked
                        //

                        // check if we clicked a shape
                        if let Some(down_on_shape) =
                            &self.mouse_previous_down_on_shape[mouse_button_index]
                        {
                            if let Some(closest_shape) =
                                self.get_closest_shape(
                                    just_clicked_mouse_position,
                                    viewport,
                                    debug_draw_3d,
                                    debug_draw_2d,
                                    GetClosestShapeCallReason::JustClicked
                                )
                            {
                                let shape = &self.shapes[closest_shape.index];
                                if closest_shape.distance
                                    < MAX_MOUSE_INTERACT_DISTANCE_FROM_SHAPE
                                    && down_on_shape.shape_id == shape.id
                                {
                                    self.shape_last_interacted = shape.id.clone();
                                    self.shape_just_clicked[mouse_button_index] =
                                        Some(EditorDraw3DShapeClickedState {
                                            mouse_position: just_clicked_mouse_position,
                                            shape_id: shape.id.clone(),
                                        });
                                }
                            }
                        }
                    }
                }
            }

            //
            // Handle mouse up = mouse is no longer down on a shape
            //
            if input_state
                .mouse_button_went_up_position(mouse_button)
                .is_some()
            {
                self.mouse_previous_down_on_shape[mouse_button_index] = None;
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
