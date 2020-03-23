use atelier_assets::importer::{typetag, SerdeImportable};
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use skulpin::skia_safe;
use legion::entity::Entity;
use ncollide2d::world::CollisionWorld;
use legion::world::World;
use ncollide2d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide2d::shape::{Ball, Cuboid};
use ncollide2d::shape::ShapeHandle;
use minimum::components::{
    PositionComponent, UniformScaleComponent, NonUniformScaleComponent, Rotation2DComponent,
};
use imgui_inspect_derive;
use minimum::math::Vec3;
use minimum::math::Vec4;
use imgui_inspect_derive::Inspect;
use legion::prelude::*;
use minimum::resources::editor::OpenedPrefabState;
use nalgebra_glm as glm;

use atelier_assets::importer as atelier_importer;
use crate::math_conversions::vec2_glam_to_glm;

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
        resources: &Resources,
        opened_prefab: &OpenedPrefabState,
        world: &World,
        entity: Entity,
    ) {
        if let Some(position) = world.get_component::<PositionComponent>(entity) {
            let mut half_extents = *self.half_extents;

            if let Some(uniform_scale) = world.get_component::<UniformScaleComponent>(entity) {
                half_extents *= uniform_scale.uniform_scale;
            }

            if let Some(non_uniform_scale) = world.get_component::<NonUniformScaleComponent>(entity)
            {
                half_extents *= *non_uniform_scale.non_uniform_scale;
            }

            let mut rotation = 0.0;
            if let Some(rotation_component) = world.get_component::<Rotation2DComponent>(entity) {
                rotation = rotation_component.rotation;
            }

            let shape_handle = ShapeHandle::new(Cuboid::new(glm::Vec2::new(
                half_extents.x(),
                half_extents.y(),
            )));

            collision_world.add(
                ncollide2d::math::Isometry::new(
                    vec2_glam_to_glm(*position.position.xy()),
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
        resources: &Resources,
        opened_prefab: &OpenedPrefabState,
        world: &World,
        entity: Entity,
    ) {
        if let Some(position) = world.get_component::<PositionComponent>(entity) {
            let mut radius = self.radius;

            if let Some(uniform_scale) = world.get_component::<UniformScaleComponent>(entity) {
                radius *= uniform_scale.uniform_scale;
            }

            //TODO: Warn if radius is 0
            let shape_handle = ShapeHandle::new(Ball::new(radius.max(0.01)));
            collision_world.add(
                ncollide2d::math::Isometry::new(vec2_glam_to_glm(*position.position.xy()), 0.0),
                shape_handle,
                CollisionGroups::new(),
                GeometricQueryType::Proximity(0.001),
                entity,
            );
        }
    }
}
