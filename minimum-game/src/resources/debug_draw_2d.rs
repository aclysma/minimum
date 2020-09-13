pub struct LineList2D {
    pub points: Vec<glam::Vec2>,
    pub color: glam::Vec4,
}

impl LineList2D {
    pub fn new(
        points: Vec<glam::Vec2>,
        color: glam::Vec4,
    ) -> Self {
        LineList2D { points, color }
    }
}

pub struct DebugDraw2DResource {
    line_lists: Vec<LineList2D>,
}

impl DebugDraw2DResource {
    pub fn new() -> Self {
        DebugDraw2DResource { line_lists: vec![] }
    }

    // Adds a single polygon
    pub fn add_line_strip(
        &mut self,
        points: Vec<glam::Vec2>,
        color: glam::Vec4,
    ) {
        // Nothing will draw if we don't have at least 2 points
        if points.len() > 1 {
            self.line_lists.push(LineList2D::new(points, color));
        }
    }

    pub fn add_line_loop(
        &mut self,
        mut points: Vec<glam::Vec2>,
        color: glam::Vec4,
    ) {
        // Nothing will draw if we don't have at least 2 points
        if points.len() > 1 {
            points.push(points[0]);
            self.add_line_strip(points, color);
        }
    }

    pub fn add_line(
        &mut self,
        p0: glam::Vec2,
        p1: glam::Vec2,
        color: glam::Vec4,
    ) {
        let points = vec![p0, p1];

        self.add_line_strip(points, color);
    }

    pub fn add_tristrip(
        &mut self,
        points: &[glam::Vec2],
        color: glam::Vec4,
    ) {
        // Nothing will draw if we don't have at least 2 points
        for index in 0..(points.len() - 2) {
            let v = vec![points[index], points[index + 1], points[index + 2]];
            self.add_line_loop(v, color);
        }
    }

    pub fn add_circle(
        &mut self,
        center: glam::Vec2,
        radius: f32,
        color: glam::Vec4,
    ) {
        let point_count = 12;

        let mut points = Vec::with_capacity(point_count);
        for index in 0..point_count {
            let fraction = (index as f32 / point_count as f32) * std::f32::consts::PI * 2.0;

            points.push(glam::Vec2::new(fraction.sin() * radius, fraction.cos() * radius) + center);
        }

        self.add_line_loop(points, color);
    }

    pub fn add_rect(
        &mut self,
        p0: glam::Vec2,
        p1: glam::Vec2,
        color: glam::Vec4,
    ) {
        let points = vec![
            p0,
            glam::vec2(p0.x(), p1.y()),
            p1,
            glam::vec2(p1.x(), p0.y()),
            p0,
        ];

        self.add_line_loop(points, color);
    }

    // Returns the draw data, leaving this object in an empty state
    pub fn take_line_lists(&mut self) -> Vec<LineList2D> {
        std::mem::replace(&mut self.line_lists, vec![])
    }

    // Recommended to call every frame to ensure that this doesn't grow unbounded
    pub fn clear(&mut self) {
        self.line_lists.clear();
    }
}
