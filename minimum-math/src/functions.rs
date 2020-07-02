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

pub fn distance_to_2d_segment_sq(
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
pub struct RayIntersectResult {
    pub t0: f32,
    pub t1: f32,
    pub c0: glam::Vec3,
    pub c1: glam::Vec3
}

pub fn ray_intersect(
    p0: glam::Vec3,
    d0: glam::Vec3,
    t0_max: f32,
    p1: glam::Vec3,
    d1: glam::Vec3,
    t1_max: f32
) -> RayIntersectResult {
    const SMALL_NUMBER : f32 = 0.00001;

    //Close enough to equivalent
    if (p1 - p0).length_squared() < SMALL_NUMBER {
        return RayIntersectResult {
            c0: p0,
            c1: p0,
            t0: 0.0,
            t1: 0.0,
        }
    }

    // Get a direction orthogonal to both rays
    let n = d0.cross(d1);
    if n.length_squared() < SMALL_NUMBER {
        // It's parallel. Determine distance between the lines.
        // https://math.stackexchange.com/questions/1347604/find-3d-distance-between-two-parallel-lines-in-simple-way
        //let distance = d1.cross(p1 - p0);

        // Use dot product to get the ideal t0 required to get as close as possible to the point on
        // the other line. The clamp between 0.0 and t0_max. Use t0 to determine c0.
        let t0 = d0.dot(p1 - p0).max(0.0).min(t0_max);
        let c0 = p0 + t0 * d0;

        // Now project p1 - c0. It's possible c0 was constrained by 0 < t0 < t0_max.
        let t1 = d1.dot(c0 - p1).max(0.0).min(t1_max);
        let c1 = p1 + t1 * d1;

        let t0 = d0.dot(c1 - p0).max(0.0).min(t0_max);
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
        let t0 = (n1.dot(p1 - p0) / d0.dot(n1)).max(0.0).min(t0_max);
        let c0 = p0 + t0 * d0;

        // Now figure out c1 based on where c0 ended up. This will factor in t0 being constrained
        //let d1_norm = d1.normalize();
        let t1 = (d1.dot(c0 - p1) / d1.length_squared()).max(0.0).min(t1_max);
        let c1 = p1 + t1 * d1;

        // Recompute t0/c0 as t1 may have been constrained
        let t0 = (d0.dot(c1 - p0) / d0.length_squared()).max(0.0).min(t0_max);
        let c0 = p0 + t0 * d0;

        RayIntersectResult {
            t0,
            t1,
            c0,
            c1
        }
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 1.0);
    assert_eq!(result.t1, 2.0);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 6.0);
    assert_eq!(result.t1, 5.0);
    assert_eq!(result.c0, glam::Vec3::new(0.0, 5.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(0.0, 5.0, 0.0));
}

#[test]
fn ray_intersect_opposite_direction() {
    let p0 = glam::Vec3::new(-1.0, -1.0, 0.0);
    let d0 = glam::Vec3::new(-1.0, -1.0, 0.0);
    let t0_max = f32::MAX;

    let p1 = glam::Vec3::new(0.0, -4.0, 0.0);
    let d1 = glam::Vec3::new(0.0, 2.0, 0.0);
    let t1_max = f32::MAX;

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 0.0);
    assert_eq!(result.t1, 1.5);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
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

    let result = ray_intersect(p0, d0, t0_max, p1, d1, t1_max);
    println!("{:#?}", result);
    assert_eq!(result.t0, 0.0);
    assert_eq!(result.t1, 2.0);
    assert_eq!(result.c0, glam::Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(result.c1, glam::Vec3::new(1.0, 0.0, 0.0));
}
