use ncollide3d::world::CollisionWorld;
use legion::*;
use legion::storage::Component;
use std::marker::PhantomData;

use crate::resources::{OpenedPrefabState, EditorStateResource};

const EDITOR_SELECTION_WORLD_MARGIN: f32 = 0.02;

/// Any selectable component must implement this trait
pub trait EditorSelectable: legion::storage::Component {
    /// When called, the implementation is expected to place shapes into the collision world
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        resources: &Resources,
        opened_prefab: &OpenedPrefabState,
        world: &World,
        entity: Entity,
    );
}

/// Any selectable component must implement this trait
pub trait EditorSelectableTransformed<T>: legion::storage::Component {
    /// When called, the implementation is expected to place shapes into the collision world
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        resources: &Resources,
        opened_prefab: &OpenedPrefabState,
        prefab_world: &World,
        prefab_entity: Entity,
        transformed_world: &World,
        transformed_entity: Entity,
        transformed_component: &T,
    );
}

/// A trait object which allows dynamic dispatch into the selection implementation
trait RegisteredEditorSelectableT: Send + Sync {
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        resources: &Resources,
        opened_prefab: &OpenedPrefabState,
        world: &World,
    );
}

/// Implements the RegisteredEditorSelectableT trait object with code that can call
/// create_editor_selection_world on T
#[derive(Default)]
struct RegisteredEditorSelectable<T> {
    phantom_data: PhantomData<T>,
}

impl<T> RegisteredEditorSelectable<T>
where
    T: EditorSelectable,
{
    fn new() -> Self {
        RegisteredEditorSelectable {
            phantom_data: Default::default(),
        }
    }
}

impl<T> RegisteredEditorSelectableT for RegisteredEditorSelectable<T>
where
    T: EditorSelectable,
{
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        resources: &Resources,
        opened_prefab: &OpenedPrefabState,
        world: &World,
    ) {
        let mut query = <(Entity, Read<T>)>::query();
        for (entity, t) in query.iter(world) {
            t.create_editor_selection_world(
                collision_world,
                resources,
                opened_prefab,
                world,
                *entity,
            );
        }
    }
}

/// Implements the RegisteredEditorSelectableT trait object with code that can call
/// create_editor_selection_world on T
#[derive(Default)]
struct RegisteredEditorSelectableTransformed<T, U> {
    phantom_data: PhantomData<(T, U)>,
}

impl<T, U> RegisteredEditorSelectableTransformed<T, U>
where
    T: EditorSelectableTransformed<U>,
{
    fn new() -> Self {
        RegisteredEditorSelectableTransformed {
            phantom_data: Default::default(),
        }
    }
}

impl<T, U> RegisteredEditorSelectableT for RegisteredEditorSelectableTransformed<T, U>
where
    T: EditorSelectableTransformed<U>,
    U: Component,
{
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        resources: &Resources,
        opened_prefab: &OpenedPrefabState,
        world: &World,
    ) {
        let mut query = <(Entity, Read<U>)>::query();
        for (world_entity, world_component) in query.iter(world) {
            if let Some(prefab_entity) = opened_prefab.world_to_prefab_mappings().get(world_entity)
            {
                let entity_ref = opened_prefab
                    .cooked_prefab()
                    .world
                    .entry_ref(*prefab_entity)
                    .unwrap();

                if let Ok(prefab_component) = entity_ref.get_component::<T>() {
                    prefab_component.create_editor_selection_world(
                        collision_world,
                        resources,
                        opened_prefab,
                        &opened_prefab.cooked_prefab().world,
                        *prefab_entity,
                        world,
                        *world_entity,
                        &*world_component,
                    );
                }
            }
        }
    }
}

#[derive(Default)]
pub struct EditorSelectRegistryBuilder {
    registered: Vec<Box<dyn RegisteredEditorSelectableT>>,
}

impl EditorSelectRegistryBuilder {
    pub fn new() -> EditorSelectRegistryBuilder {
        Self::default()
    }

    /// Adds a type to the registry, which allows components of these types to receive a callback
    /// to insert shapes into the collision world used for selection
    pub fn register<T: EditorSelectable>(mut self) -> Self {
        self.registered
            .push(Box::new(RegisteredEditorSelectable::<T>::new()));
        self
    }

    pub fn register_transformed<T: EditorSelectableTransformed<U>, U: Component>(mut self) -> Self {
        self.registered.push(Box::new(
            RegisteredEditorSelectableTransformed::<T, U>::new(),
        ));
        self
    }

    pub fn build(self) -> EditorSelectRegistry {
        EditorSelectRegistry {
            registered: self.registered,
        }
    }
}

pub struct EditorSelectRegistry {
    registered: Vec<Box<dyn RegisteredEditorSelectableT>>,
}

impl EditorSelectRegistry {
    pub fn create_empty_editor_selection_world(&self) -> CollisionWorld<f32, Entity> {
        CollisionWorld::<f32, Entity>::new(EDITOR_SELECTION_WORLD_MARGIN)
    }

    /// Produces a collision world that includes shapes associated with entities
    pub fn create_editor_selection_world(
        &self,
        resources: &Resources,
        world: &World,
    ) -> CollisionWorld<f32, Entity> {
        let mut collision_world = self.create_empty_editor_selection_world();

        if let Some(opened_prefab) = resources
            .get::<EditorStateResource>()
            .unwrap()
            .opened_prefab()
        {
            for r in &self.registered {
                r.create_editor_selection_world(
                    &mut collision_world,
                    resources,
                    &*opened_prefab,
                    &world,
                );
            }
        }

        collision_world
    }
}
