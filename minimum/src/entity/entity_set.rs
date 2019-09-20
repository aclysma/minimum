use std::prelude::v1::*;

use crate::{slab, ComponentStorage};
use slab::GenSlab;

use super::Entity;
use super::EntityFactory;
use super::EntityHandle;
use super::EntityRef;
use crate::resource;

use crate::component;
use component::Component;
use component::ComponentRegistry;

use super::PendingDeleteComponent;

/// Manages adding/removing/retrieving entities
pub struct EntitySet {
    slab: GenSlab<Entity>,
    component_registry: ComponentRegistry,
}

impl EntitySet {
    /// Create a new empty set of entities using the given component registry
    pub fn new(component_registry: ComponentRegistry) -> Self {
        EntitySet {
            slab: GenSlab::new(),
            component_registry,
        }
    }

    /// Allocates an entity immediately. Using EntityFactory and prototypes instead is preferred.
    /// However, this is exposed so that you can write your own logic for this, or for cases where
    /// you need to use the entity immediately.
    ///
    /// Returns a handle to the entity
    pub fn allocate(&mut self) -> EntityHandle {
        let handle = self.slab.allocate(Entity::new());
        self.slab
            .get_mut(&handle)
            .unwrap()
            .set_handle(handle.clone());
        handle
    }

    /// Allocates an entity immediately. Using EntityFactory and prototypes instead is preferred.
    /// However, this is exposed so that you can write your own logic for this, or for cases where
    /// you need to use the entity immediately.
    ///
    /// Returns an EntityRef, which can be used to add/remove components
    pub fn allocate_get(&mut self) -> EntityRef {
        let handle = self.slab.allocate(Entity::new());
        let entity = self.slab.get_mut(&handle).unwrap();
        entity.set_handle(handle.clone());
        EntityRef::new(entity)
    }

    /// Free the given entity during the next EntitySet::update.
    pub fn enqueue_free(
        &self,
        entity_handle: &EntityHandle,
        delete_components: &mut <PendingDeleteComponent as Component>::Storage,
    ) {
        if delete_components.exists(entity_handle) {
            return;
        }

        self.get_entity_ref(entity_handle)
            .unwrap()
            .add_component(delete_components, PendingDeleteComponent {})
            .unwrap();
    }

    /// Get the number of entities that exist
    pub fn entity_count(&self) -> usize {
        self.slab.count()
    }

    /// Get an EntityRef for the given entity. Returns None if the entity cannot be found. The EntityRef
    /// allows for adding/removing components from the entity
    pub fn get_entity_ref(&self, entity_handle: &EntityHandle) -> Option<EntityRef> {
        let e = self.slab.get(entity_handle)?;
        Some(EntityRef::new(e))
    }

    /// Destroy all entities and their components immediately
    pub fn clear(&mut self, resource_map: &resource::ResourceMap) {
        let entity_handles: Vec<_> = self.iter().map(|x| x.handle()).collect();
        self.do_flush_free(resource_map, entity_handles.as_slice());
    }

    /// Immediately handle all deferred free calls. Usually you would just call update(), which
    /// would call this for you.
    pub fn flush_free(&mut self, resource_map: &resource::ResourceMap) {
        let entity_handles: Vec<_> = {
            let delete_components =
                resource_map.fetch_mut::<<PendingDeleteComponent as Component>::Storage>();
            delete_components.iter(&self).map(|x| x.0).collect()
        };

        self.do_flush_free(resource_map, &entity_handles);
    }

    fn do_flush_free(
        &mut self,
        resource_map: &resource::ResourceMap,
        entity_handles: &[EntityHandle],
    ) {
        self.component_registry
            .on_entities_free(resource_map, entity_handles);

        for pending_delete in entity_handles {
            self.slab.free(pending_delete);
        }
    }

    /// Immediately allocate all entities that were enqueued in the EntityFactory. Normally, you would just call
    /// update() which would call this for you.
    pub fn flush_creates(&mut self, resource_map: &resource::ResourceMap) {
        resource_map
            .fetch_mut::<EntityFactory>()
            .flush_creates(resource_map, self);
        self.component_registry.on_flush_creates(resource_map, self);
    }

    /// Call once a frame to handle any deferred entity create/free calls
    pub fn update(&mut self, resource_map: &resource::ResourceMap) {
        self.flush_free(resource_map);
        self.flush_creates(resource_map);
    }

    /// Iterate across all entities
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.slab.iter()
    }

    // This is used interally to convert a component index to the entity handle. It can be dangerous
    // to use since it's possible for a component to use the wrong "version" of an instance. (For example
    // if entity in slot 5 is destroyed and created, and a component attached to the "old" entity in slot
    // 5 tried to get the entity handle of whatever is in slot 5, it could end up getting associated with
    // the wrong entity)
    pub(crate) fn upgrade_index_to_handle(&self, index: u32) -> EntityHandle {
        self.slab.upgrade_index_to_handle(index).unwrap()
    }
}
