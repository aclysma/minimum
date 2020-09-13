pub fn normal_to_xy(normal: glam::Vec3) -> (glam::Vec3, glam::Vec3) {
    if normal.dot(glam::Vec3::unit_z()).abs() > 0.9999 {
        // Can't cross the Z axis with the up vector, so special case that here
        (glam::Vec3::unit_x(), glam::Vec3::unit_y())
    } else {
        let x_dir = normal.cross(glam::Vec3::unit_z());
        let y_dir = x_dir.cross(normal);
        (x_dir, y_dir)
    }
}

#[derive(Debug)]
pub struct DistanceTo2dSegmentResult {
    pub t: f32,
    pub closest_point: glam::Vec2,
    pub distance_sq: f32,
}

pub fn point_segment_intersect_2d(
    test_point: glam::Vec2,
    p0: glam::Vec2,
    p1: glam::Vec2,
) -> DistanceTo2dSegmentResult {
    let p0_to_p1 = p1 - p0;
    let p0_to_p1_length = p0_to_p1.length();

    // Early out in case of extremely short segment, get distance to midpoint
    if p0_to_p1_length < 0.001 {
        let midpoint = p0 * 0.5 + p1 * 0.5;
        return DistanceTo2dSegmentResult {
            t: 0.5,
            closest_point: midpoint,
            distance_sq: (test_point - midpoint).length_squared(),
        };
    }

    // distance to the infinite line described by the points
    let p0_to_p1_normalized = p0_to_p1 / p0_to_p1_length;
    let p0_to_test_point = test_point - p0;

    // find closest point to the line
    let distance_along_segment = glam::Vec2::dot(p0_to_p1_normalized, p0_to_test_point);
    if distance_along_segment <= 0.0 {
        // early out, test_point is closer to p0 than any other part of the line
        return DistanceTo2dSegmentResult {
            t: 0.0,
            closest_point: p0,
            distance_sq: (test_point - p0).length_squared(),
        };
    }

    if distance_along_segment >= p0_to_p1_length {
        // test_point is closer to p1 than any other part of the line
        DistanceTo2dSegmentResult {
            t: 1.0,
            closest_point: p1,
            distance_sq: (test_point - p1).length_squared(),
        }
    } else {
        // the closest point on the segment to test_point is between p0 and p1
        // let distance_to_line = glam::Vec2::dot(normal, p0_to_test_point);
        let t = distance_along_segment / p0_to_p1_length;
        let closest_point = p0 + p0_to_p1 * t;

        DistanceTo2dSegmentResult {
            t,
            closest_point,
            distance_sq: (test_point - closest_point).length_squared(),
        }
    }
}

pub fn distance_to_circle_sq(
    test_point: glam::Vec2,
    center: glam::Vec2,
    radius: f32,
) -> f32 {
    ((test_point - center).length_squared() - (radius * radius)).abs()
}

pub fn distance_to_3d_segment_sq(
    test_point: glam::Vec3,
    p0: glam::Vec3,
    p1: glam::Vec3,
) -> f32 {
    let p0_to_p1 = p1 - p0;

    // Early out in case of extremely short segment, get distance to midpoint
    let segment_length_sq = p0_to_p1.length_squared();
    if segment_length_sq < 0.001 {
        let midpoint = p0 + (p0_to_p1 / 2.0);
        return (test_point - midpoint).length_squared();
    }

    // Get a normal from p0 towards p1
    let segment_length = segment_length_sq.sqrt();
    let tangent = p0_to_p1 / segment_length;

    // Create a vector from p0 to the test point, we will project it onto the segment
    let p0_to_test_point = test_point - p0;

    // The projection will return a parametric floating point value, clamp it [0.0, 1.0] to pick the
    // point on the segment closest to the test point
    let t_unclamped = glam::Vec3::dot(tangent, p0_to_test_point) / segment_length;
    let t = t_unclamped.max(0.0).min(1.0);

    let closest_point_on_segment = p0 + (p0_to_p1 * t);
    (closest_point_on_segment - test_point).length_squared()
}

//TODO: Need to implement ray to segment

pub fn distance_to_sphere_sq(
    test_point: glam::Vec3,
    center: glam::Vec3,
    radius: f32,
) -> f32 {
    ((test_point - center).length_squared() - (radius * radius)).abs()
}

#[derive(Debug)]
pub struct Segment {
    pub p0: glam::Vec3,
    pub p1: glam::Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct NormalizedRay {
    pub origin: glam::Vec3,
    pub dir: glam::Vec3,
    pub length: f32,
}

pub enum PlaneIntersectResult {
    NoIntersection,
    LineContainedWithinPlane,
    Intersect(glam::Vec3),
}

pub fn line_plane_intersect_3d(
    line_position: glam::Vec3,
    line_dir: glam::Vec3,
    plane_origin: glam::Vec3,
    plane_normal_dir: glam::Vec3,
) -> PlaneIntersectResult {
    let line_dot_normal = line_dir.dot(plane_normal_dir);
    let line_to_plane = plane_origin - line_position;
    if line_dot_normal.abs() < std::f32::EPSILON {
        if line_to_plane.dot(plane_normal_dir) < std::f32::EPSILON {
            PlaneIntersectResult::NoIntersection
        } else {
            PlaneIntersectResult::LineContainedWithinPlane
        }
    } else {
        let d = line_to_plane.dot(plane_normal_dir) / line_dot_normal;
        let intersection = line_position + line_dir * d;
        PlaneIntersectResult::Intersect(intersection)
    }
}

#[derive(Debug)]
pub struct RayIntersectResult {
    pub t0: f32,
    pub t1: f32,
    pub c0: glam::Vec3,
    pub c1: glam::Vec3,
}

pub fn ray_ray_intersect_3d(
    r0: NormalizedRay,
    r1: NormalizedRay,
    t0_max: f32,
    t1_max: f32,
) -> RayIntersectResult {
    ray_intersect_internal(
        r0.origin,
        r0.dir * r0.length,
        0.0,
        t0_max,
        r1.origin,
        r1.dir * r1.length,
        0.0,
        t1_max,
    )
}

pub fn line_ray_intersect_3d(
    r0: NormalizedRay,
    r1: NormalizedRay,
    t1_max: f32,
) -> RayIntersectResult {
    ray_intersect_internal(
        r0.origin,
        r0.dir * r0.length,
        std::f32::MIN,
        std::f32::MAX,
        r1.origin,
        r1.dir * r1.length,
        0.0,
        t1_max,
    )
}

pub fn line_line_intersect_3d(
    r0: NormalizedRay,
    r1: NormalizedRay,
) -> RayIntersectResult {
    ray_intersect_internal(
        r0.origin,
        r0.dir * r0.length,
        std::f32::MIN,
        std::f32::MAX,
        r1.origin,
        r1.dir * r1.length,
        std::f32::MIN,
        std::f32::MAX,
    )
}

pub fn ray_intersect_internal(
    p0: glam::Vec3,
    d0: glam::Vec3,
    t0_min: f32,
    t0_max: f32,
    p1: glam::Vec3,
    d1: glam::Vec3,
    t1_min: f32,
    t1_max: f32,
) -> RayIntersectResult {
    const SMALL_NUMBER: f32 = 0.00001;

    //Close enough to equivalent
    if (p1 - p0).length_squared() < SMALL_NUMBER {
        return RayIntersectResult {
            c0: p0,
            c1: p0,
            t0: 0.0,
            t1: 0.0,
        };
    }

    // Get a direction orthogonal to both rays
    let n = d0.cross(d1);
    if n.length_squared() < SMALL_NUMBER {
        // It's parallel. Determine distance between the lines.
        // https://math.stackexchange.com/questions/1347604/find-3d-distance-between-two-parallel-lines-in-simple-way
        //let distance = d1.cross(p1 - p0);

        // Use dot product to get the ideal t0 required to get as close as possible to the point on
        // the other line. The clamp between 0.0 and t0_max. Use t0 to determine c0.
        let t0 = d0.dot(p1 - p0).max(t0_min).min(t0_max);
        let c0 = p0 + t0 * d0;

        // Now project p1 - c0. It's possible c0 was constrained by 0 < t0 < t0_max.
        let t1 = d1.dot(c0 - p1).max(t1_min).min(t1_max);
        let c1 = p1 + t1 * d1;

        let t0 = d0.dot(c1 - p0).max(t0_min).min(t0_max);
        let c0 = p0 + t0 * d0;

        RayIntersectResult {
            //distance,
            c0,
            c1,
            t0,
            t1,
        }
    } else {
        // based on https://en.wikipedia.org/wiki/Skew_lines#Nearest_Points
        let n1 = d1.cross(n);

        // Calculate c0 assuming that t1 will not be constrained and that t1 = (c0 - p1).dot(d1)
        let t0 = (n1.dot(p1 - p0) / d0.dot(n1)).max(t0_min).min(t0_max);
        let c0 = p0 + t0 * d0;

        // Now figure out c1 based on where c0 ended up. This will factor in t0 being constrained
        //let d1_norm = d1.normalize();
        let t1 = (d1.dot(c0 - p1) / d1.length_squared())
            .max(t1_min)
            .min(t1_max);
        let c1 = p1 + t1 * d1;

        // Recompute t0/c0 as t1 may have been constrained
        let t0 = (d0.dot(c1 - p0) / d0.length_squared())
            .max(t0_min)
            .min(t0_max);
        let c0 = p0 + t0 * d0;

        RayIntersectResult { t0, t1, c0, c1 }
    }
}

#[test]
fn ray_intersect_same_origin() {
    let p0 = glam::Vec3::new(0.0, 0.0, 0.0);
    let d0 = glam::Vec3::new(2.0, 0.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(0.0, 0.0, 0.0);
    let d1 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 0.0);
    assert_eq!(result.t1, 0.0);
    assert_eq!(result.c0, glam::Vec3::zero());
    assert_eq!(result.c1, glam::Vec3::zero());
}

#[test]
fn ray_intersect_given_points_closest_orthogonal() {
    let p0 = glam::Vec3::new(0.0, 0.0, 0.0);
    let d0 = glam::Vec3::new(2.0, 0.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(0.0, 0.0, 1.0);
    let d1 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 0.0);
    assert_eq!(result.t1, 0.0);
    assert_eq!(result.c0, glam::Vec3::zero());
    assert_eq!(result.c1, glam::Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn ray_intersect_orthogonal_non_intersect() {
    let p0 = glam::Vec3::new(-2.0, 0.0, 0.0);
    let d0 = glam::Vec3::new(2.0, 0.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(0.0, -4.0, 1.0);
    let d1 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 1.0);
    assert_eq!(result.t1, 2.0);
    assert_eq!(result.c0, glam::Vec3::zero());
    assert_eq!(result.c1, glam::Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn ray_intersect_orthogonal_intersect() {
    let p0 = glam::Vec3::new(-1.0, 0.0, 0.0);
    let d0 = glam::Vec3::new(1.0, 0.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(0.0, -4.0, 0.0);
    let d1 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 1.0);
    assert_eq!(result.t1, 2.0);
    assert_eq!(result.c0, glam::Vec3::zero());
    assert_eq!(result.c1, glam::Vec3::zero());
}

#[test]
fn ray_intersect_non_orthogonal_non_intersect() {
    let p0 = glam::Vec3::new(-1.0, -1.0, 0.0);
    let d0 = glam::Vec3::new(1.0, 1.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(0.0, -4.0, 0.0);
    let d1 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_approx_eq!(result.t0, 1.0);
    assert_approx_eq!(result.t1, 2.0);
    assert_eq!(result.c0, glam::Vec3::zero());
    assert_eq!(result.c1, glam::Vec3::zero());
}

#[test]
fn ray_intersect_non_power2() {
    let p0 = glam::Vec3::new(0.0, -25.0, 0.0);
    let d0 = glam::Vec3::new(0.0, 5.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(-5.0, 0.0, 0.0);
    let d1 = glam::Vec3::new(1.0, 1.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_approx_eq!(result.t0, 6.0);
    assert_approx_eq!(result.t1, 5.0);
    assert_approx_eq!(
        0.0,
        (result.c0 - glam::Vec3::new(0.0, 5.0, 0.0)).length_squared()
    );
    assert_approx_eq!(
        0.0,
        (result.c1 - glam::Vec3::new(0.0, 5.0, 0.0)).length_squared()
    );
}

#[test]
fn ray_intersect_opposite_direction() {
    let p0 = glam::Vec3::new(-1.0, -1.0, 0.0);
    let d0 = glam::Vec3::new(-1.0, -1.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(0.0, -4.0, 0.0);
    let d1 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_approx_eq!(result.t0, 0.0);
    assert_approx_eq!(result.t1, 1.5);
    assert_eq!(result.c0, glam::Vec3::new(-1.0, -1.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(0.0, -1.0, 0.0));
}

#[test]
fn ray_intersect_opposite_direction_rev() {
    let p0 = glam::Vec3::new(0.0, -4.0, 0.0);
    let d0 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(-1.0, -1.0, 0.0);
    let d1 = glam::Vec3::new(-1.0, -1.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 1.5);
    assert_eq!(result.t1, 0.0);
    assert_eq!(result.c0, glam::Vec3::new(0.0, -1.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(-1.0, -1.0, 0.0));
}

#[test]
fn ray_intersect_distance_limited() {
    let p0 = glam::Vec3::new(-1.0, -1.0, 0.0);
    let d0 = glam::Vec3::new(1.0, 1.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(0.0, -4.0, 0.0);
    let d1 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t1_max = 1.0;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 0.0);
    assert_eq!(result.t1, 1.0);
    assert_eq!(result.c0, glam::Vec3::new(-1.0, -1.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(0.0, -2.0, 0.0));
}

#[test]
fn ray_intersect_distance_limited_rev() {
    let p0 = glam::Vec3::new(0.0, -4.0, 0.0);
    let d0 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t0_max = 1.0;

    let p1 = glam::Vec3::new(-1.0, -1.0, 0.0);
    let d1 = glam::Vec3::new(1.0, 1.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 1.0);
    assert_eq!(result.t1, 0.0);
    assert_eq!(result.c0, glam::Vec3::new(0.0, -2.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(-1.0, -1.0, 0.0));
}

#[test]
fn ray_intersect_parallel_towards() {
    let p0 = glam::Vec3::new(-1.0, 0.0, 0.0);
    let d0 = glam::Vec3::new(1.0, 0.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(1.0, 0.0, 0.0);
    let d1 = glam::Vec3::new(-1.0, 0.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 2.0);
    assert_eq!(result.t1, 0.0);
    assert_eq!(result.c0, glam::Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn ray_intersect_parallel_away() {
    let p0 = glam::Vec3::new(-1.0, 0.0, 0.0);
    let d0 = glam::Vec3::new(-1.0, 0.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(1.0, 0.0, 0.0);
    let d1 = glam::Vec3::new(1.0, 0.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 0.0);
    assert_eq!(result.t1, 0.0);
    assert_eq!(result.c0, glam::Vec3::new(-1.0, 0.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn ray_intersect_parallel_same_dir() {
    let p0 = glam::Vec3::new(-1.0, 0.0, 0.0);
    let d0 = glam::Vec3::new(1.0, 0.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(1.0, 0.0, 0.0);
    let d1 = glam::Vec3::new(1.0, 0.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 2.0);
    assert_eq!(result.t1, 0.0);
    assert_eq!(result.c0, glam::Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn ray_intersect_parallel_same_dir_rev() {
    let p0 = glam::Vec3::new(1.0, 0.0, 0.0);
    let d0 = glam::Vec3::new(1.0, 0.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(-1.0, 0.0, 0.0);
    let d1 = glam::Vec3::new(1.0, 0.0, 0.0);
    let t1_max = f32::MAX;

    let r0 = NormalizedRay {
        origin: p0,
        dir: d0.normalize(),
        length: d0.length(),
    };
    let r1 = NormalizedRay {
        origin: p1,
        dir: d1.normalize(),
        length: d1.length(),
    };

    let result = ray_ray_intersect_3d(r0, r1, t0_max, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 0.0);
    assert_eq!(result.t1, 2.0);
    assert_eq!(result.c0, glam::Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn point_segment_intersect_2d_0deg() {
    let test_point = glam::Vec2::new(100.0, 0.0);
    let p0 = glam::Vec2::new(0.0, 100.0);
    let p1 = glam::Vec2::new(200.0, 100.0);

    let result = point_segment_intersect_2d(test_point, p0, p1);
    let dot = (result.closest_point - test_point).dot(p1 - p0);

    println!("{:#?}", result);
    println!("dot {}", dot);
    assert_eq!(result.t, 0.5);
    assert_eq!(result.closest_point, glam::Vec2::new(100.0, 100.0));
    assert_eq!(result.distance_sq, 10000.0);
    assert!(dot.abs() < 0.0001);
}

#[test]
fn point_segment_intersect_2d_45deg() {
    let test_point = glam::Vec2::new(100.0, 0.0);
    let p0 = glam::Vec2::new(100.0, 100.0);
    let p1 = glam::Vec2::new(200.0, 0.0);

    let result = point_segment_intersect_2d(test_point, p0, p1);
    let dot = (result.closest_point - test_point).dot(p1 - p0);

    println!("{:#?}", result);
    println!("dot {}", dot);
    assert_eq!(result.t, 0.5);
    assert_eq!(result.closest_point, glam::Vec2::new(150.0, 50.0));
    assert_eq!(result.distance_sq, 5000.0);
    assert!(dot.abs() < 0.0001);
}

#[test]
fn point_segment_intersect_2d_30deg() {
    let test_point = glam::Vec2::new(100.0, 0.0);
    let p0 = glam::Vec2::new(100.0, 300.0);
    let p1 = glam::Vec2::new(500.0, 0.0);

    let result = point_segment_intersect_2d(test_point, p0, p1);
    let dot = (result.closest_point - test_point).dot(p1 - p0);

    println!("{:#?}", result);
    println!("dot {}", dot);
    assert_eq!(result.t, 0.36);
    assert_eq!(result.closest_point, glam::Vec2::new(244.0, 192.0));
    assert_eq!(result.distance_sq, 57600.0);
    assert!(dot.abs() < 0.0001);
}

#[test]
fn point_segment_intersect_2d_60deg() {
    let test_point = glam::Vec2::new(100.0, 0.0);
    let p0 = glam::Vec2::new(100.0, 400.0);
    let p1 = glam::Vec2::new(400.0, 0.0);

    let result = point_segment_intersect_2d(test_point, p0, p1);
    let dot = (result.closest_point - test_point).dot(p1 - p0);

    println!("{:#?}", result);
    println!("dot {}", dot);
    assert_eq!(result.t, 0.64);
    assert_eq!(result.closest_point, glam::Vec2::new(292.0, 144.0));
    assert_eq!(result.distance_sq, 57600.0);
    assert!(dot.abs() < 0.0001);
}

#[test]
fn point_segment_intersect_2d_90deg() {
    let test_point = glam::Vec2::new(100.0, 0.0);
    let p0 = glam::Vec2::new(200.0, 100.0);
    let p1 = glam::Vec2::new(200.0, -100.0);

    let result = point_segment_intersect_2d(test_point, p0, p1);
    let dot = (result.closest_point - test_point).dot(p1 - p0);

    println!("{:#?}", result);
    println!("dot {}", dot);
    assert_eq!(result.t, 0.5);
    assert_eq!(result.closest_point, glam::Vec2::new(200.0, 0.0));
    assert_eq!(result.distance_sq, 10000.0);
    assert!(dot.abs() < 0.0001);
}
