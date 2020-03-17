use legion_prefab::ComponentRegistration;
use prefab_format::ComponentTypeUuid;
use legion::storage::ComponentTypeId;
use std::collections::HashMap;
use crate::resources::AssetResource;
use legion_transaction::{CopyCloneImpl, SpawnCloneImpl, SpawnCloneImplHandlerSet, SpawnInto};
use legion::prelude::Resources;
use legion::storage::{Component, ComponentStorage};
use legion::index::ComponentIndex;
use legion::prelude::*;
use std::ops::Range;
use std::mem::MaybeUninit;

pub struct ComponentRegistryBuilder {
    components: HashMap<ComponentTypeId, ComponentRegistration>,
    components_by_uuid: HashMap<ComponentTypeUuid, ComponentRegistration>,
    spawn_handler_set: SpawnCloneImplHandlerSet,
}

impl ComponentRegistryBuilder {
    pub fn new() -> Self {
        ComponentRegistryBuilder {
            components: Default::default(),
            components_by_uuid: Default::default(),
            spawn_handler_set: SpawnCloneImplHandlerSet::new(),
        }
    }

    pub fn auto_register_components(mut self) -> Self {
        let comp_registrations = legion_prefab::iter_component_registrations();
        use std::iter::FromIterator;

        for registration in comp_registrations {
            self = self.register_component(registration);
        }

        self
    }

    pub fn register_component(
        mut self,
        registration: &ComponentRegistration,
    ) -> Self {
        self.components
            .insert(registration.component_type_id(), registration.clone());
        self.components_by_uuid
            .insert(*registration.uuid(), registration.clone());
        self
    }

    pub fn add_spawn_mapping_into<FromT: Component + Clone + Into<IntoT>, IntoT: Component>(
        mut self
    ) -> Self {
        self.spawn_handler_set.add_mapping_into::<FromT, IntoT>();
        self
    }

    pub fn add_spawn_mapping<FromT: Component + Clone + SpawnInto<IntoT>, IntoT: Component>(
        mut self
    ) -> Self {
        self.spawn_handler_set.add_mapping::<FromT, IntoT>();
        self
    }

    pub fn add_spawn_mapping_closure<FromT, IntoT, F>(
        &mut self,
        clone_fn: F,
    ) where
        FromT: Component,
        IntoT: Component,
        F: Fn(
                &World,                    // src_world
                &ComponentStorage,         // src_component_storage
                Range<ComponentIndex>,     // src_component_storage_indexes
                &Resources,                // resources
                &[Entity],                 // src_entities
                &[Entity],                 // dst_entities
                &[FromT],                  // src_data
                &mut [MaybeUninit<IntoT>], // dst_data
            ) + Send
            + Sync
            + 'static,
    {
        self.spawn_handler_set.add_mapping_closure(clone_fn);
    }

    pub fn build(self) -> ComponentRegistry {
        ComponentRegistry {
            components: self.components,
            components_by_uuid: self.components_by_uuid,
            spawn_handler_set: self.spawn_handler_set,
        }
    }
}

pub struct ComponentRegistry {
    components: HashMap<ComponentTypeId, ComponentRegistration>,
    components_by_uuid: HashMap<ComponentTypeUuid, ComponentRegistration>,
    spawn_handler_set: SpawnCloneImplHandlerSet,
}

impl ComponentRegistry {
    pub fn components(&self) -> &HashMap<ComponentTypeId, ComponentRegistration> {
        &self.components
    }

    pub fn components_by_uuid(&self) -> &HashMap<ComponentTypeUuid, ComponentRegistration> {
        &self.components_by_uuid
    }

    pub fn copy_clone_impl<'a>(&'a self) -> CopyCloneImpl<'a> {
        CopyCloneImpl::new(&self.components)
    }

    pub fn spawn_clone_impl<'a, 'b>(
        &'a self,
        resources: &'b Resources,
    ) -> SpawnCloneImpl<'a, 'a, 'b> {
        SpawnCloneImpl::new(&self.spawn_handler_set, &self.components, resources)
    }
}
