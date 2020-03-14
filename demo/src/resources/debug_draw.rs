pub struct LineList {
    pub points: Vec<glam::Vec2>,
    pub color: glam::Vec4,
}

impl LineList {
    pub fn new(
        points: Vec<glam::Vec2>,
        color: glam::Vec4,
    ) -> Self {
        LineList { points, color }
    }
}

pub struct DebugDrawResource {
    line_lists: Vec<LineList>,
}

impl DebugDrawResource {
    pub fn new() -> Self {
        DebugDrawResource { line_lists: vec![] }
    }

    // Adds a single polygon
    pub fn add_polygon(
        &mut self,
        mut points: Vec<glam::Vec2>,
        color: glam::Vec4,
    ) {
        // Nothing will draw if we don't have at least 2 points
        if points.len() > 1 {
            points.push(points[0].clone());
            self.line_lists.push(LineList::new(points, color));
        }
    }

    pub fn add_tristrip(
        &mut self,
        points: &Vec<glam::Vec2>,
        color: glam::Vec4,
    ) {
        // Nothing will draw if we don't have at least 2 points
        for index in 0..(points.len() - 2) {
            let v = vec![points[index], points[index + 1], points[index + 2]];
            self.add_polygon(v, color);
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

        self.add_polygon(points, color);
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

        self.add_polygon(points, color);
    }

    pub fn add_line(
        &mut self,
        p0: glam::Vec2,
        p1: glam::Vec2,
        color: glam::Vec4,
    ) {
        let points = vec![p0, p1];

        self.add_polygon(points, color);
    }

    // Returns the draw data, leaving this object in an empty state
    pub fn take_line_lists(&mut self) -> Vec<LineList> {
        std::mem::replace(&mut self.line_lists, vec![])
    }

    // Recommended to call every frame to ensure that this doesn't grow unbounded
    pub fn clear(&mut self) {
        self.line_lists.clear();
    }
}
