use crate::slab;
use slab::GenSlab;

use super::Entity;
use super::EntityFactory;
use super::EntityHandle;
use super::EntityRef;
use crate::resource;

use crate::component;
use component::Component;
use component::ComponentFactory;
use component::ComponentFreeHandler;
use component::ComponentPrototype;
use component::ComponentRegistry;

use super::PendingDeleteComponent;

pub struct EntitySet {
    slab: GenSlab<Entity>,
    component_registry: ComponentRegistry,
}

impl EntitySet {
    pub fn new() -> Self {
        EntitySet {
            slab: GenSlab::new(),
            component_registry: ComponentRegistry::new(),
        }
    }

    pub fn register_component_type<T: Component>(&mut self) {
        self.component_registry.register_component::<T>();
    }

    //TODO: Improve this API
    pub fn register_component_type_with_free_handler<
        T: Component,
        F: ComponentFreeHandler<T> + 'static,
    >(
        &mut self,
    ) {
        self.component_registry
            .register_component_type_with_free_handler::<T, F>();
    }

    pub fn register_component_factory<P: ComponentPrototype, F: ComponentFactory<P>>(&mut self) {
        self.component_registry.register_component_factory::<P, F>();
    }

    pub fn allocate(&mut self) -> EntityHandle {
        let handle = self.slab.allocate(Entity::new());
        self.slab
            .get_mut(&handle)
            .unwrap()
            .set_handle(handle.clone());
        handle
    }

    pub fn allocate_get(&mut self) -> EntityRef {
        let handle = self.slab.allocate(Entity::new());
        let entity = self.slab.get_mut(&handle).unwrap();
        entity.set_handle(handle.clone());
        EntityRef::new(entity, handle)
    }

    pub fn enqueue_free(
        &self,
        entity_handle: &EntityHandle,
        delete_components: &mut <PendingDeleteComponent as Component>::Storage,
    ) {
        self.get_entity_ref(entity_handle)
            .unwrap()
            .add_component(delete_components, PendingDeleteComponent {});
    }

    pub fn entity_count(&self) -> usize {
        self.slab.count()
    }

    pub fn get_entity_ref(&self, entity_handle: &EntityHandle) -> Option<EntityRef> {
        let handle = (*entity_handle).clone();
        let e = self.slab.get(entity_handle)?;
        Some(EntityRef::new(e, handle))
    }

    pub fn clear(&mut self, world: &resource::World) {
        let entity_handles: Vec<_> = self.iter().map(|x| x.handle()).collect();
        self.do_flush_free(world, entity_handles.as_slice());
    }

    pub fn flush_free(&mut self, world: &resource::World) {
        let entity_handles: Vec<_> = {
            let delete_components =
                world.fetch_mut::<<PendingDeleteComponent as Component>::Storage>();
            delete_components.iter(&self).map(|x| x.0).collect()
        };

        self.do_flush_free(world, &entity_handles);
    }

    pub fn do_flush_free(&mut self, world: &resource::World, entity_handles: &[EntityHandle]) {
        self.component_registry
            .on_entities_free(world, entity_handles);

        for pending_delete in entity_handles {
            self.slab.free(pending_delete);
        }
    }

    pub fn flush_creates(&mut self, world: &resource::World) {
        world
            .fetch_mut::<EntityFactory>()
            .flush_creates(world, self);
        self.component_registry.on_flush_creates(world, self);
    }

    pub fn visit_components(&self, world: &resource::World, entity_handles: &[EntityHandle]) {
        self.component_registry.visit_components(world, entity_handles);
    }

    pub fn update(&mut self, world: &resource::World) {
        self.flush_free(world);
        self.flush_creates(world);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.slab.iter()
    }

    pub fn upgrade_index_to_handle(&self, index: u32) -> EntityHandle {
        self.slab.upgrade_index_to_handle(index).unwrap()
    }
}
