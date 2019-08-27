use minimum::component::VecComponentStorage;
use minimum::component::{ComponentCreateQueueFlushListener, ComponentStorage};
use minimum::Component;
use minimum::ComponentFactory;
use minimum::ComponentPrototype;
use minimum::EntityHandle;
use minimum::EntitySet;
use minimum::ResourceMap;

use nphysics2d::object::ColliderHandle;

use crate::framework::FrameworkComponentPrototype;
use named_type::NamedType;
use ncollide2d::shape::ShapeHandle;
use ncollide2d::world::{CollisionGroups, GeometricQueryType};
use std::collections::VecDeque;

#[derive(Clone, NamedType)]
pub struct EditorShapeComponent {
    shape_handle: ShapeHandle<f32>,
    collider_handle: ColliderHandle,
}

impl EditorShapeComponent {
    pub fn new(shape_handle: ShapeHandle<f32>, collider_handle: ColliderHandle) -> Self {
        EditorShapeComponent {
            shape_handle,
            collider_handle,
        }
    }

    pub fn collider_handle(&self) -> &ColliderHandle {
        &self.collider_handle
    }

    pub fn shape_handle(&self) -> &ShapeHandle<f32> {
        &self.shape_handle
    }
}

impl minimum::Component for EditorShapeComponent {
    type Storage = VecComponentStorage<Self>;
}

//
// The free handler ensures that when an entity is destroyed, its body components get cleaned up
//
pub struct EditorShapeComponentFreeHandler {}

impl minimum::component::ComponentFreeHandler<EditorShapeComponent>
    for EditorShapeComponentFreeHandler
{
    fn on_entities_free(
        resource_map: &minimum::ResourceMap,
        entity_handles: &[minimum::EntityHandle],
        storage: &mut <EditorShapeComponent as Component>::Storage,
    ) {
        let mut editor_collision_world =
            resource_map.fetch_mut::<crate::resources::EditorCollisionWorld>();
        let physics_world: &mut ncollide2d::world::CollisionWorld<f32, EntityHandle> =
            editor_collision_world.world_mut();

        for entity_handle in entity_handles {
            if let Some(c) = storage.get_mut(&entity_handle) {
                physics_world.remove(&[c.collider_handle]);
            }
        }
    }
}

//
// Creates a component
//
#[derive(Clone, NamedType)]
pub struct EditorShapeComponentPrototype {
    shape_handle: ShapeHandle<f32>,
}

impl EditorShapeComponentPrototype {
    pub fn new(shape_handle: ShapeHandle<f32>) -> Self {
        EditorShapeComponentPrototype { shape_handle }
    }
}

impl ComponentPrototype for EditorShapeComponentPrototype {
    type Factory = EditorShapeComponentFactory;
}

impl FrameworkComponentPrototype for EditorShapeComponentPrototype {}

//
// Factory for PhysicsBody components
//
pub struct EditorShapeComponentFactory {
    prototypes: VecDeque<(EntityHandle, EditorShapeComponentPrototype)>,
}

impl EditorShapeComponentFactory {
    pub fn new() -> Self {
        EditorShapeComponentFactory {
            prototypes: VecDeque::new(),
        }
    }
}

impl ComponentFactory<EditorShapeComponentPrototype> for EditorShapeComponentFactory {
    fn enqueue_create(
        &mut self,
        entity_handle: &EntityHandle,
        prototype: &EditorShapeComponentPrototype,
    ) {
        self.prototypes
            .push_back((entity_handle.clone(), prototype.clone()));
    }
}

impl ComponentCreateQueueFlushListener for EditorShapeComponentFactory {
    fn flush_creates(&mut self, resource_map: &ResourceMap, entity_set: &EntitySet) {
        if self.prototypes.is_empty() {
            return;
        }

        let mut collision_world =
            resource_map.fetch_mut::<crate::resources::EditorCollisionWorld>();
        let mut storage = resource_map.fetch_mut::<<EditorShapeComponent as Component>::Storage>();
        for (entity_handle, data) in self.prototypes.drain(..) {
            if let Some(entity) = entity_set.get_entity_ref(&entity_handle) {
                let collider = collision_world.world_mut().add(
                    nalgebra::Isometry2::new(glm::vec2(0.0, 0.0), 0.0),
                    data.shape_handle.clone(),
                    CollisionGroups::new(),
                    GeometricQueryType::Proximity(0.001),
                    entity_handle,
                );

                entity.add_component(
                    &mut *storage,
                    EditorShapeComponent::new(data.shape_handle, collider.handle()),
                );
            }
        }
    }
}
