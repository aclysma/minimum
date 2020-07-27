use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use skulpin::skia_safe;
use legion::entity::Entity;
use ncollide3d::world::CollisionWorld;
use legion::world::World;
use ncollide3d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide3d::shape::{Ball, Cuboid};
use ncollide3d::shape::ShapeHandle;
use minimum::components::{TransformComponent, TransformComponentDef};
use imgui_inspect_derive;
use minimum::math::Vec3;
use minimum::math::Vec4;
use imgui_inspect_derive::Inspect;
use legion::prelude::*;
use minimum::resources::editor::OpenedPrefabState;
use nalgebra_glm as glm;

use crate::math_conversions::{vec2_glam_to_glm, vec3_glam_to_glm, quat_glam_to_glm};

// A utility struct to describe color for a skia shape
#[derive(Clone, Copy, Debug, Serialize, Deserialize, SerdeDiff, PartialEq, Inspect, Default)]
pub struct PaintDef {
    #[serde_diff(opaque)]
    pub color: Vec4,
    pub stroke_width: f32,
}

pub struct Paint(pub std::sync::Mutex<skia_safe::Paint>);
unsafe impl Send for Paint {}
unsafe impl Sync for Paint {}

impl From<PaintDef> for Paint {
    fn from(from: PaintDef) -> Self {
        let color = skia_safe::Color4f::new(
            from.color.x(),
            from.color.y(),
            from.color.z(),
            from.color.w(),
        );

        let mut paint = skia_safe::Paint::new(color, None);
        paint.set_anti_alias(true);
        paint.set_style(skia_safe::paint::Style::Stroke);
        paint.set_stroke_width(from.stroke_width);

        Paint(std::sync::Mutex::new(paint))
    }
}

//
// Draw a box at the component's current location. Will be affected by scale, if the scale component
// exists
//
#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Inspect, Default)]
#[uuid = "c05e5c27-58ca-4d68-b825-b20f67fdaf37"]
pub struct DrawSkiaBoxComponentDef {
    #[serde_diff(opaque)]
    pub half_extents: Vec3,
    pub paint: PaintDef,
}

legion_prefab::register_component_type!(DrawSkiaBoxComponentDef);

pub struct DrawSkiaBoxComponent {
    pub half_extents: Vec3,
    pub paint: Paint,
}

impl From<DrawSkiaBoxComponentDef> for DrawSkiaBoxComponent {
    fn from(from: DrawSkiaBoxComponentDef) -> Self {
        DrawSkiaBoxComponent {
            half_extents: from.half_extents,
            paint: from.paint.into(),
        }
    }
}

impl minimum::editor::EditorSelectable for DrawSkiaBoxComponent {
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        _resources: &Resources,
        _opened_prefab: &OpenedPrefabState,
        world: &World,
        entity: Entity,
    ) {
        if let Some(transform) = world.get_component::<TransformComponentDef>(entity) {
            let mut half_extents = *self.half_extents;

            half_extents *= transform.scale();

            let shape_handle = ShapeHandle::new(Cuboid::new(vec3_glam_to_glm(half_extents)));

            //TODO: This might be wrong
            let rotation = quat_glam_to_glm(transform.rotation_quat());
            let rotation = nalgebra::UnitQuaternion::from_quaternion(rotation);
            collision_world.add(
                ncollide3d::math::Isometry::from_parts(
                    nalgebra::Translation::from(vec3_glam_to_glm(*transform.position)),
                    rotation,
                ),
                shape_handle,
                CollisionGroups::new(),
                GeometricQueryType::Proximity(0.001),
                entity,
            );
        }
    }
}

//
// Draw a circle at the component's current location. Will be affected by scale, if the scale
// component exists
//
#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Inspect, Default)]
#[uuid = "e47f9943-d5bf-4e1b-9601-13e47d7b737c"]
pub struct DrawSkiaCircleComponentDef {
    pub radius: f32,
    pub paint: PaintDef,
}

legion_prefab::register_component_type!(DrawSkiaCircleComponentDef);

pub struct DrawSkiaCircleComponent {
    pub radius: f32,
    pub paint: Paint,
}

impl From<DrawSkiaCircleComponentDef> for DrawSkiaCircleComponent {
    fn from(from: DrawSkiaCircleComponentDef) -> Self {
        let c = DrawSkiaCircleComponent {
            radius: from.radius,
            paint: from.paint.into(),
        };
        c
    }
}

impl minimum::editor::EditorSelectable for DrawSkiaCircleComponent {
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        _resources: &Resources,
        _opened_prefab: &OpenedPrefabState,
        world: &World,
        entity: Entity,
    ) {
        if let Some(transform) = world.get_component::<TransformComponentDef>(entity) {
            let mut radius = self.radius;
            radius *= transform.uniform_scale();

            //TODO: Warn if radius is 0
            let shape_handle = ShapeHandle::new(Ball::new(radius.max(0.01)));
            //TODO: This might be wrong
            let rotation = quat_glam_to_glm(transform.rotation_quat());
            let rotation = nalgebra::UnitQuaternion::from_quaternion(rotation);
            collision_world.add(
                ncollide3d::math::Isometry::from_parts(
                    nalgebra::Translation::from(vec3_glam_to_glm(transform.position())),
                    rotation,
                ),
                shape_handle,
                CollisionGroups::new(),
                GeometricQueryType::Proximity(0.001),
                entity,
            );
        }
    }
}
