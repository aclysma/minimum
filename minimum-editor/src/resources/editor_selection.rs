use ncollide3d::world::CollisionWorld;
use ncollide3d::bounding_volume::AABB;
use legion::*;

use std::collections::HashSet;
use std::collections::HashMap;
use std::sync::Arc;

use crate::resources::EditorStateResource;
use crate::EditorSelectRegistry;

enum SelectionOp {
    Add(Vec<Entity>),
    Remove(Vec<Entity>),
    Set(Vec<Entity>),
    Clear,
}

pub struct EditorSelectionResource {
    registry: Arc<EditorSelectRegistry>,
    editor_selection_world: CollisionWorld<f32, Entity>,

    // These are entities in the world
    selected_entities: HashSet<Entity>,

    pending_selection_ops: Vec<SelectionOp>,
}

impl EditorSelectionResource {
    pub fn new(registry: EditorSelectRegistry) -> Self {
        let editor_selection_world = registry.create_empty_editor_selection_world();

        EditorSelectionResource {
            registry: Arc::new(registry),
            editor_selection_world,
            selected_entities: Default::default(),
            pending_selection_ops: Default::default(),
        }
    }

    pub fn create_editor_selection_world(
        resources: &Resources,
        world: &World,
    ) -> CollisionWorld<f32, Entity> {
        let registry = { resources.get::<Self>().unwrap().registry.clone() };

        registry.create_editor_selection_world(resources, world)
    }

    pub fn set_editor_selection_world(
        &mut self,
        editor_selection_world: CollisionWorld<f32, Entity>,
    ) {
        self.editor_selection_world = editor_selection_world;
    }

    pub fn editor_selection_world(&mut self) -> &CollisionWorld<f32, Entity> {
        &self.editor_selection_world
    }

    pub fn selected_entities(&self) -> &HashSet<Entity> {
        &self.selected_entities
    }

    pub fn selected_entity_aabbs(&mut self) -> HashMap<Entity, Option<AABB<f32>>> {
        Self::get_entity_aabbs(&self.selected_entities, &mut self.editor_selection_world)
    }

    pub fn enqueue_add_to_selection(
        &mut self,
        entities: Vec<Entity>,
    ) {
        log::info!("add entities {:?} from selection", entities);
        self.pending_selection_ops.push(SelectionOp::Add(entities));
    }

    pub fn enqueue_remove_from_selection(
        &mut self,
        entities: Vec<Entity>,
    ) {
        log::info!("remove entities {:?} to selection", entities);
        self.pending_selection_ops
            .push(SelectionOp::Remove(entities));
    }

    pub fn enqueue_clear_selection(&mut self) {
        log::info!("Clear selection");
        self.pending_selection_ops.push(SelectionOp::Clear);
    }

    pub fn enqueue_set_selection(
        &mut self,
        selected_entities: Vec<Entity>,
    ) {
        log::trace!("Selected entities: {:?}", selected_entities);
        self.pending_selection_ops
            .push(SelectionOp::Set(selected_entities));
    }

    pub fn is_entity_selected(
        &self,
        entity: Entity,
    ) -> bool {
        self.selected_entities.contains(&entity)
    }

    pub fn process_selection_ops(
        &mut self,
        _editor_state: &mut EditorStateResource,
        _world: &mut World,
    ) -> bool {
        let ops: Vec<_> = self.pending_selection_ops.drain(..).collect();

        let mut changed = false;
        for op in ops {
            changed |= match op {
                SelectionOp::Add(entities) => {
                    let mut changed = false;
                    for e in entities {
                        changed |= self.selected_entities.insert(e);
                    }

                    changed
                }
                SelectionOp::Remove(entities) => {
                    let mut changed = false;
                    for e in entities {
                        changed |= self.selected_entities.remove(&e);
                    }

                    changed
                }
                SelectionOp::Clear => {
                    if self.selected_entities.len() > 0 {
                        self.selected_entities.clear();
                        true
                    } else {
                        false
                    }
                }
                SelectionOp::Set(entities) => {
                    self.selected_entities = entities.iter().map(|x| *x).collect();
                    true
                }
            }
        }

        changed
    }

    // The main reason for having such a specific function here is that it's awkward for an external
    // caller to borrow entities and world seperately
    fn get_entity_aabbs(
        entities: &HashSet<Entity>,
        world: &CollisionWorld<f32, Entity>,
    ) -> HashMap<Entity, Option<AABB<f32>>> {
        let mut entity_aabbs = HashMap::new();
        for e in entities {
            entity_aabbs.insert(*e, None);
        }

        for (_, shape) in world.collision_objects() {
            let _entry =
                entity_aabbs
                    .entry(*shape.data())
                    .and_modify(|aabb: &mut Option<AABB<f32>>| {
                        let mut new_aabb = shape.shape().aabb(shape.position());
                        if let Some(existing_aabb) = aabb {
                            use ncollide3d::bounding_volume::BoundingVolume;
                            new_aabb.merge(existing_aabb);
                        };

                        *aabb = Some(new_aabb);
                    });
        }

        entity_aabbs
    }
}
