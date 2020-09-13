use minimum::resources::ViewportResource;

// this is a virtual coordinate system
// top-left: (0, 0)
// bottom-right: (600 * aspect_ratio, 600) where aspect_ratio is window_width / window_height
fn calculate_screen_space_matrix(
    _viewport_size_in_pixels: glam::Vec2,
    view_half_extents: glam::Vec2,
) -> glam::Mat4 {
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::from([0.0, 0.0, 5.0]),
        glam::Vec3::from([0.0, 0.0, 0.0]),
        glam::Vec3::from([0.0, 1.0, 0.0]),
    );

    let projection = glam::Mat4::orthographic_rh(
        0.0,
        view_half_extents.x() * 2.0,
        view_half_extents.y() * 2.0,
        0.0,
        -100.0,
        100.0,
    );

    projection * view
}

// this is a virtual coordinate system where h = 600 and w = 600 * aspect_ratio where
// aspect_ratio is window_width / window_height
// top-left: (-w/2, -h/2)
// bottom-right: (w/2, h/2)
fn calculate_world_space_view_matrix(
    _viewport_size_in_pixels: glam::Vec2,
    _position: glam::Vec3,
    _view_half_extents: glam::Vec2,
) -> glam::Mat4 {
    glam::Mat4::look_at_rh(
        glam::Vec3::from([0.0, 0.0, 5.0]),
        glam::Vec3::from([0.0, 0.0, 0.0]),
        glam::Vec3::from([0.0, 1.0, 0.0]),
    )
}

fn calculate_world_space_proj_matrix(
    _viewport_size_in_pixels: glam::Vec2,
    position: glam::Vec3,
    view_half_extents: glam::Vec2,
) -> glam::Mat4 {
    glam::Mat4::orthographic_rh(
        position.x() - view_half_extents.x(),
        position.x() + view_half_extents.x(),
        position.y() + view_half_extents.y(),
        position.y() - view_half_extents.y(),
        -100.0,
        100.0,
    )
}

pub fn update_viewport(
    viewport: &mut ViewportResource,
    viewport_size_in_pixels: glam::Vec2,
    camera_position: glam::Vec2,
    x_half_extents: f32,
) -> glam::Vec2 {
    let y_half_extents =
        x_half_extents / (viewport_size_in_pixels.x() / viewport_size_in_pixels.y());

    let view_half_extents = glam::Vec2::new(x_half_extents, y_half_extents);

    let camera_position = glam::Vec3::new(camera_position.x(), camera_position.y(), 0.0);

    viewport.set_viewport_size_in_pixels(viewport_size_in_pixels);
    viewport.set_screen_space_view(calculate_screen_space_matrix(
        viewport_size_in_pixels,
        view_half_extents,
    ));

    viewport.set_world_space_view(
        calculate_world_space_proj_matrix(
            viewport_size_in_pixels,
            camera_position,
            view_half_extents,
        ),
        calculate_world_space_view_matrix(
            viewport_size_in_pixels,
            camera_position,
            view_half_extents,
        ),
        camera_position,
    );

    view_half_extents
}
