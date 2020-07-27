use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use nphysics2d::object::DefaultBodyHandle;
use legion_prefab::SpawnFrom;
use minimum::math::Vec3;
use crate::resources::PhysicsResource;
use minimum::resources::editor::OpenedPrefabState;
use legion::prelude::*;
use std::ops::Range;
use legion::storage::ComponentStorage;
use imgui;
use imgui_inspect_derive::Inspect;
use ncollide2d::shape::ShapeHandle as ShapeHandle2d;
use ncollide3d::shape::ShapeHandle as ShapeHandle3d;
use ncollide3d::shape::Ball as Ball3d;
use ncollide3d::shape::Cuboid as Cuboid3d;
use ncollide2d::shape::Ball as Ball2d;
use ncollide2d::shape::Cuboid as Cuboid2d;
use ncollide3d::pipeline::{CollisionGroups, GeometricQueryType};
use legion::index::ComponentIndex;
use legion_transaction::iter_components_in_storage;
use nalgebra_glm as glm;

use minimum::components::{
    TransformComponentDef
};
use ncollide3d::world::CollisionWorld;

use crate::math_conversions::{vec2_glam_to_glm, vec3_glam_to_glm, quat_glam_to_glm};

//
// Add a ball rigid body
//
#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Inspect, Default)]
#[uuid = "fa518c0a-a65a-44c8-9d35-3f4f336b4de4"]
pub struct RigidBodyBallComponentDef {
    pub radius: f32,
    pub is_static: bool,
}

legion_prefab::register_component_type!(RigidBodyBallComponentDef);

#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Inspect, Default)]
#[uuid = "36df3006-a5ad-4997-9ccc-0860f49195ad"]
pub struct RigidBodyBoxComponentDef {
    #[serde_diff(opaque)]
    pub half_extents: Vec3,
    pub is_static: bool,
}

legion_prefab::register_component_type!(RigidBodyBoxComponentDef);

pub struct RigidBodyComponent {
    pub handle: DefaultBodyHandle,
    delete_body_tx: crossbeam_channel::Sender<DefaultBodyHandle>,
}

impl Drop for RigidBodyComponent {
    fn drop(&mut self) {
        self.delete_body_tx.send(self.handle).unwrap();
    }
}

fn transform_shape_to_rigid_body(
    physics: &mut PhysicsResource,
    into: &mut std::mem::MaybeUninit<RigidBodyComponent>,
    src_transform: Option<&TransformComponentDef>,
    shape_handle: ShapeHandle2d<f32>,
    is_static: bool,
) {
    let position = if let Some(position) = src_transform {
        position.position
    } else {
        Vec3::zero()
    };

    let mut collider_offset = Vec3::zero();

    // Build the rigid body.
    let rigid_body_handle = if is_static {
        *collider_offset += *position;
        physics.bodies.insert(nphysics2d::object::Ground::new())
    } else {
        physics.bodies.insert(
            nphysics2d::object::RigidBodyDesc::new()
                .translation(vec2_glam_to_glm(position.xy().into()))
                .build(),
        )
    };

    // Build the collider.
    let collider = nphysics2d::object::ColliderDesc::new(shape_handle.clone())
        .density(1.0)
        .translation(vec2_glam_to_glm(*collider_offset.xy()))
        .build(nphysics2d::object::BodyPartHandle(rigid_body_handle, 0));

    // Insert the collider to the body set.
    physics.colliders.insert(collider);

    *into = std::mem::MaybeUninit::new(RigidBodyComponent {
        handle: rigid_body_handle,
        delete_body_tx: physics.delete_body_tx().clone(),
    })
}

impl SpawnFrom<RigidBodyBallComponentDef> for RigidBodyComponent {
    fn spawn_from(
        _src_world: &World,
        src_component_storage: &ComponentStorage,
        src_component_storage_indexes: Range<ComponentIndex>,
        resources: &Resources,
        _src_entities: &[Entity],
        _dst_entities: &[Entity],
        from: &[RigidBodyBallComponentDef],
        into: &mut [std::mem::MaybeUninit<Self>],
    ) {
        let mut physics = resources.get_mut::<PhysicsResource>().unwrap();

        let position_components = iter_components_in_storage::<TransformComponentDef>(
            src_component_storage,
            src_component_storage_indexes.clone(),
        );

        for (src_transform, from, into) in izip!(
            position_components,
            from,
            into
        ) {
            let mut radius = from.radius;
            if let Some(transform) = src_transform {
                radius *= transform.uniform_scale();
            }

            //TODO: Warn if radius is 0
            let shape_handle = ShapeHandle2d::new(Ball2d::new(radius.max(0.01)));
            transform_shape_to_rigid_body(
                &mut physics,
                into,
                src_transform,
                shape_handle,
                from.is_static,
            );
        }
    }
}

impl minimum::editor::EditorSelectableTransformed<RigidBodyComponent>
    for RigidBodyBallComponentDef
{
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        _resources: &Resources,
        _opened_prefab: &OpenedPrefabState,
        prefab_world: &World,
        prefab_entity: Entity,
        _transformed_world: &World,
        transformed_entity: Entity,
        _transformed_component: &RigidBodyComponent,
    ) {
        if let Some(transform) = prefab_world.get_component::<TransformComponentDef>(prefab_entity) {
            let mut radius = self.radius;
            radius *= transform.uniform_scale();

            let shape_handle = ShapeHandle3d::new(Ball3d::new(radius.max(0.01)));

            //TODO: This might be wrong
            let rotation = quat_glam_to_glm(transform.rotation_quat());
            let rotation = nalgebra::UnitQuaternion::from_quaternion(rotation);
            collision_world.add(
                ncollide3d::math::Isometry::from_parts(
                    nalgebra::Translation::from(vec3_glam_to_glm(*transform.position)),
                    rotation
                ),
                shape_handle,
                CollisionGroups::new(),
                GeometricQueryType::Proximity(0.001),
                transformed_entity,
            );
        }
    }
}

impl SpawnFrom<RigidBodyBoxComponentDef> for RigidBodyComponent {
    fn spawn_from(
        _src_world: &World,
        src_component_storage: &ComponentStorage,
        src_component_storage_indexes: Range<ComponentIndex>,
        resources: &Resources,
        _src_entities: &[Entity],
        _dst_entities: &[Entity],
        from: &[RigidBodyBoxComponentDef],
        into: &mut [std::mem::MaybeUninit<Self>],
    ) {
        let mut physics = resources.get_mut::<PhysicsResource>().unwrap();

        let transform_components = iter_components_in_storage::<TransformComponentDef>(
            src_component_storage,
            src_component_storage_indexes.clone(),
        );

        for (src_transform, from, into) in izip!(
            transform_components,
            from,
            into
        ) {
            let mut half_extents = *from.half_extents;

            if let Some(transform) = src_transform {
                half_extents *= transform.scale();
            }

            let shape_handle = ShapeHandle2d::new(Cuboid2d::new(glm::Vec2::new(
                half_extents.x(),
                half_extents.y(),
            )));
            transform_shape_to_rigid_body(
                &mut physics,
                into,
                src_transform,
                shape_handle,
                from.is_static,
            );
        }
    }
}

impl minimum::editor::EditorSelectableTransformed<RigidBodyComponent> for RigidBodyBoxComponentDef {
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        _resources: &Resources,
        _opened_prefab: &OpenedPrefabState,
        prefab_world: &World,
        prefab_entity: Entity,
        _transformed_world: &World,
        transformed_entity: Entity,
        _transformed_component: &RigidBodyComponent,
    ) {
        if let Some(transform) = prefab_world.get_component::<TransformComponentDef>(prefab_entity) {
            let mut half_extents = *self.half_extents;
            half_extents *= transform.scale();

            let shape_handle = ShapeHandle3d::new(Cuboid3d::new(glm::Vec3::new(
                half_extents.x(),
                half_extents.y(),
                0.0
            )));

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
                transformed_entity,
            );
        }
    }
}
