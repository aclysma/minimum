use super::DebugDraw;
use super::InputManager;
use super::RenderState;

fn distance_to_segment_sq(test_point: glm::Vec2, p0: glm::Vec2, p1: glm::Vec2) -> f32 {
    let p0_to_p1 = p1 - p0;

    // Early out in case of extremely short segment, get distance to midpoint
    if glm::length2(&p0_to_p1) < 0.01 {
        let midpoint = p0 + (p0_to_p1 / 2.0);
        return glm::distance2(&test_point, &midpoint);
    }

    // Get "tangent" and "normal" of the segment
    let tangent = glm::normalize(&p0_to_p1);
    let normal = glm::vec2(tangent.y, -tangent.x);

    // distance to the infinite line described by the points
    let p0_to_test_point = test_point - p0;

    // find closest point to the line
    let distance_along_segment = glm::dot(&tangent, &(p0_to_test_point));
    if distance_along_segment <= 0.0 {
        // early out, test_point is closer to p0 than any other part of the line
        return glm::distance2(&test_point, &p0)
    }

    let fraction_along_segment = (distance_along_segment * distance_along_segment) / glm::length2(&p0_to_p1);
    if fraction_along_segment >= 1.0 {
        // test_point is closer to p1 than any other part of the line
        glm::distance2(&test_point, &p1)
    } else {
        // the closest point on the segment to test_point is between p0 and p1
        let distance_to_line = glm::dot(&normal, &p0_to_test_point);
        f32::abs(distance_to_line * distance_to_line)
    }
}

fn distance_to_circle_outline_sq(test_point: glm::Vec2, center: glm::Vec2, radius: f32) -> f32 {
    (glm::distance2(&test_point, &center) - (radius * radius)).abs()
}

struct Line {
    p0: glm::Vec2,
    p1: glm::Vec2
}

struct CircleOutline {
    center: glm::Vec2,
    radius: f32
}

enum Shape {
    Line(Line),
    CircleOutline(CircleOutline)
}

struct ShapeWithId {
    id: String,
    shape: Shape
}

impl ShapeWithId {
    fn new_line(
        id: String,
        p0: glm::Vec2,
        p1: glm::Vec2
    ) -> Self {
        ShapeWithId {
            id,
            shape: Shape::Line(Line {
                p0,
                p1
            })
        }
    }

    fn new_circle_outline(
        id: String,
        center: glm::Vec2,
        radius: f32
    ) -> Self {
        ShapeWithId {
            id,
            shape: Shape::CircleOutline(CircleOutline {
                center,
                radius
            })
        }
    }
}

struct ClosestShapeInfo {
    id: String,
    distance_sq: f32
}

pub struct EditorDraw {
    shapes: Vec<ShapeWithId>,
    closest_line_info: ClosestShapeInfo
}

impl EditorDraw {
    pub fn new() -> Self {
        EditorDraw {
            shapes: vec![],
            closest_line_info: ClosestShapeInfo {
                id: "".to_string(),
                distance_sq: std::f32::MAX
            }
        }
    }

    pub fn line(&mut self, id: &str, debug_draw: &mut DebugDraw, p0: glm::Vec2, p1: glm::Vec2, mut color: glm::Vec4) {
        if self.closest_line_info.id == id && self.closest_line_info.distance_sq < (50.0 * 50.0) {
            color = glm::vec4(1.0, 0.0, 0.0, 1.0);
        }

        debug_draw.add_line(p0, p1, color);
        self.shapes.push(ShapeWithId::new_line(id.to_string(), p0, p1));
    }

    pub fn circle_outline(&mut self, id: &str, debug_draw: &mut DebugDraw, center: glm::Vec2, radius: f32, mut color: glm::Vec4) {
        if self.closest_line_info.id == id && self.closest_line_info.distance_sq < (50.0 * 50.0) {
            color = glm::vec4(1.0, 0.0, 0.0, 1.0);
        }

        debug_draw.add_circle(center, radius, color);
        self.shapes.push(ShapeWithId::new_circle_outline(id.to_string(), center, radius));
    }

    pub fn update(&mut self, input_manager: &InputManager, render_state: &RenderState) {
        // See if the user interacted with anything. If they did, then cache it. User code would need to
        // check this and possibly check against other clickable things (like if an object in the editor was clicked.)
        // We likely need a measure of depth if we draw in the 3D world in a way that can be occluded.

        let mouse_position = input_manager.mouse_position();

        let mut closest_shape_index = None;
        let mut closest_distance_sq = std::f32::MAX;

        let index = 0;
        for i in 0..self.shapes.len() {
            let shape = &self.shapes[i];

            let distance_sq = match &shape.shape {
                Shape::Line(line) => {
                    distance_to_segment_sq(
                        mouse_position,
                        render_state.world_space_to_ui_space(line.p0),
                        render_state.world_space_to_ui_space(line.p1))
                },
                Shape::CircleOutline(circle) => {
                    // This is an odd kludge, but we want to work in ui space. However, the radius in ui space won't match the radius in
                    // world space.
                    let position_on_outline = circle.center + glm::vec2(circle.radius, 0.0);
                    let scaled_center = render_state.world_space_to_ui_space(circle.center);
                    let scaled_position_on_outline = render_state.world_space_to_ui_space(position_on_outline);
                    let scaled_radius = f32::abs(scaled_position_on_outline.x - scaled_center.x);

                    distance_to_circle_outline_sq(
                        mouse_position,
                        scaled_center,
                        scaled_radius)
                }
            };

            if distance_sq < closest_distance_sq {
                closest_distance_sq = distance_sq;
                closest_shape_index = Some(i);
            }

            //println!(" shaped {} is {} away", shape.id, distance_sq);
        }

        if let Some(closest_shape_index) = closest_shape_index {
            self.closest_line_info.id.clear();
            self.closest_line_info.id.push_str(&self.shapes[closest_shape_index].id);
            self.closest_line_info.distance_sq = closest_distance_sq;
        } else {
            self.closest_line_info.id.clear();
            self.closest_line_info.distance_sq = std::f32::MAX;
        }

        //println!("closest line is {}, {} away", self.closest_line_info.id, self.closest_line_info.distance_sq.sqrt());

        self.shapes.clear();
    }

    pub fn is_dragged(input_manager: &InputManager, id: &str) {

    }
}