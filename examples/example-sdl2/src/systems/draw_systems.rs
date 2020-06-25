use legion::prelude::*;

use skulpin::{skia_safe};

use minimum::components::{
    PositionComponent, UniformScaleComponent, NonUniformScaleComponent, Rotation2DComponent,
};
use minimum_skulpin::components::DrawSkiaBoxComponent;
use minimum_skulpin::components::DrawSkiaCircleComponent;

use minimum_skulpin::resources::CanvasDrawResource;
use minimum::resources::{CameraResource, ViewportResource, DebugDrawResource, ViewportSize};

use example_shared::resources::FpsTextResource;
use minimum_sdl2::resources::Sdl2WindowResource;

pub fn draw() -> Box<dyn Schedulable> {
    // Copy the data from physics rigid bodies into position components
    SystemBuilder::new("draw")
        .write_resource::<CanvasDrawResource>()
        .read_resource::<FpsTextResource>()
        .write_resource::<CameraResource>()
        .write_resource::<ViewportResource>()
        .write_resource::<DebugDrawResource>()
        .read_resource::<Sdl2WindowResource>()
        .with_query(<(
            Read<PositionComponent>,
            Read<DrawSkiaBoxComponent>,
            TryRead<UniformScaleComponent>,
            TryRead<NonUniformScaleComponent>,
            TryRead<Rotation2DComponent>,
        )>::query())
        .with_query(<(
            Read<PositionComponent>,
            Read<DrawSkiaCircleComponent>,
            TryRead<UniformScaleComponent>,
            TryRead<Rotation2DComponent>,
        )>::query())
        .build(
            |_,
             world,
             (
                draw_context,
                fps_text,
                camera_state,
                viewport_state,
                debug_draw,
                sdl2_window_resource,
            ),
             (draw_boxes_query, draw_circles_query)| {
                draw_context.with_canvas(|canvas, coordinate_system_helper| {
                    let drawable_size = sdl2_window_resource.drawable_size();
                    let camera_position = camera_state.position;

                    let viewport_size =
                        ViewportSize::new(drawable_size.width, drawable_size.height);

                    viewport_state.update(
                        viewport_size,
                        camera_position,
                        camera_state.x_half_extents,
                    );

                    let half_extents = viewport_state.view_half_extents();

                    coordinate_system_helper
                        .use_visible_range(
                            canvas,
                            skia_safe::Rect {
                                left: -half_extents.x() + camera_position.x(),
                                right: half_extents.x() + camera_position.x(),
                                top: half_extents.y() + camera_position.y(),
                                bottom: -half_extents.y() + camera_position.y(),
                            },
                            skia_safe::matrix::ScaleToFit::Center,
                        )
                        .unwrap();

                    // Generally would want to clear data every time we draw
                    canvas.clear(skia_safe::Color::from_argb(0, 0, 0, 255));

                    // Draw all the boxes
                    for (pos, skia_box, uniform_scale, non_uniform_scale, rotation) in
                        draw_boxes_query.iter(world)
                    {
                        let mut half_extents = *skia_box.half_extents;
                        if let Some(uniform_scale) = uniform_scale {
                            half_extents *= uniform_scale.uniform_scale;
                        }

                        if let Some(non_uniform_scale) = non_uniform_scale {
                            half_extents *= *non_uniform_scale.non_uniform_scale;
                        }

                        let paint = skia_box.paint.0.lock().unwrap();

                        let rotation_in_degrees = if let Some(rotation) = rotation {
                            rotation.rotation * 180.0 / std::f32::consts::PI
                        } else {
                            0.0
                        };

                        canvas.save();
                        canvas.rotate(
                            rotation_in_degrees,
                            Some(skia_safe::Point::new(pos.position.x(), pos.position.y())),
                        );

                        canvas.draw_rect(
                            skia_safe::Rect {
                                left: pos.position.x() - half_extents.x(),
                                right: pos.position.x() + half_extents.x(),
                                top: pos.position.y() - half_extents.y(),
                                bottom: pos.position.y() + half_extents.y(),
                            },
                            &paint,
                        );

                        canvas.restore();
                    }

                    // Draw all the circles
                    for (pos, skia_circle, uniform_scale, _rotation) in
                        draw_circles_query.iter(world)
                    {
                        let mut scale = 1.0;
                        if let Some(uniform_scale) = uniform_scale {
                            scale *= uniform_scale.uniform_scale;
                        }

                        let paint = skia_circle.paint.0.lock().unwrap();
                        canvas.draw_circle(
                            skia_safe::Point::new(pos.position.x(), pos.position.y()),
                            skia_circle.radius * scale,
                            &paint,
                        );
                    }

                    // Debug draw
                    for line_list in debug_draw.take_line_lists() {
                        if line_list.points.len() < 2 {
                            continue;
                        }

                        let paint = skia_safe::Paint::new(
                            skia_safe::Color4f::new(
                                line_list.color.x(),
                                line_list.color.y(),
                                line_list.color.z(),
                                line_list.color.w(),
                            ),
                            None,
                        );

                        let from = line_list.points[0];
                        let mut from = skia_safe::Point::new(from.x(), from.y());
                        for i in 1..line_list.points.len() {
                            let to = line_list.points[i];
                            let to = skia_safe::Point::new(to.x(), to.y());
                            canvas.draw_line(from, to, &paint);
                            from = to;
                        }
                    }

                    debug_draw.clear();

                    // Switch to using logical screen-space coordinates
                    coordinate_system_helper.use_logical_coordinates(canvas);

                    //
                    // Draw FPS text
                    //
                    let mut text_paint =
                        skia_safe::Paint::new(skia_safe::Color4f::new(1.0, 1.0, 0.0, 1.0), None);
                    text_paint.set_anti_alias(true);
                    text_paint.set_style(skia_safe::paint::Style::StrokeAndFill);
                    text_paint.set_stroke_width(1.0);

                    let mut font = skia_safe::Font::default();
                    font.set_size(20.0);
                    //canvas.draw_str(self.fps_text.clone(), (50, 50), &font, &text_paint);
                    canvas.draw_str(fps_text.fps_text.clone(), (50, 50), &font, &text_paint);
                });
            },
        )
}
