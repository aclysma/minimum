use minimum::entity::EntityPrototype;
use minimum::{Component, ComponentPrototype};

use minimum::EntityRef;
use minimum::ResourceMap;

use std::sync::Arc;
use std::sync::Mutex;

use crate::components::PersistentEntityComponent;

#[cfg(feature = "editor")]
use crate::components::editor::EditorShapeComponentPrototype;
#[cfg(feature = "editor")]
use crate::select::SelectRegistry;

pub trait FrameworkComponentPrototype {
    fn component_type() -> std::any::TypeId;
}

pub trait FrameworkComponentPrototypeDyn: minimum::component::ComponentPrototypeDyn + mopa::Any {
    fn component_type(&self) -> std::any::TypeId;
}

impl<T> FrameworkComponentPrototypeDyn for T
where T : FrameworkComponentPrototype + ComponentPrototype {
    fn component_type(&self) -> std::any::TypeId {
        T::component_type()
    }
}

mopafy!(FrameworkComponentPrototypeDyn);

pub struct FrameworkEntityPrototypeInner {
    path: std::path::PathBuf,
    component_prototypes: Vec<Box<dyn FrameworkComponentPrototypeDyn>>,
}

impl FrameworkEntityPrototypeInner {
    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }

    pub fn component_prototypes(&self) -> &Vec<Box<dyn FrameworkComponentPrototypeDyn>> {
        &self.component_prototypes
    }

    pub fn component_prototypes_mut<'a: 'b, 'b>(
        &'a mut self,
    ) -> &'b mut Vec<Box<dyn FrameworkComponentPrototypeDyn>> {
        &mut self.component_prototypes
    }

    pub fn find_component_prototype<T : FrameworkComponentPrototypeDyn>(&self) -> Option<&T> {
        for p in &self.component_prototypes {
            match p.downcast_ref::<T>() {
                Some(downcast) => return Some(downcast),
                _ => {}
            }
        }

        None
    }

    pub fn find_component_prototype_mut<T : FrameworkComponentPrototypeDyn>(&mut self) -> Option<&mut T> {
        for p in &mut self.component_prototypes {
            match p.downcast_mut::<T>() {
                Some(downcast) => return Some(downcast),
                _ => {}
            }
        }

        None
    }

    pub fn find_component_prototype_by_component_type_id(&self, type_id: std::any::TypeId) -> Option<&dyn FrameworkComponentPrototypeDyn> {
        for p in &self.component_prototypes {
            if p.component_type() == type_id {
                return Some(&**p)
            }
        }

        None
    }
}

#[derive(Clone)]
pub enum FrameworkEntityPersistencePolicy {
    // Saved to disk and is recreated on level reset
    Persistent,

    // Is destroyed on level reset (i.e. spawned at runtime)
    Transient,
}

#[derive(Clone)]
pub struct FrameworkEntityPrototype {
    inner: Arc<Mutex<FrameworkEntityPrototypeInner>>,
    persistence_policy: FrameworkEntityPersistencePolicy,
}

impl FrameworkEntityPrototype {
    pub fn new(
        path: std::path::PathBuf,
        persistence_policy: FrameworkEntityPersistencePolicy,
        component_prototypes: Vec<Box<dyn FrameworkComponentPrototypeDyn>>,
    ) -> Self {
        FrameworkEntityPrototype {
            inner: Arc::new(Mutex::new(FrameworkEntityPrototypeInner {
                path,
                component_prototypes,
            })),
            persistence_policy,
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<FrameworkEntityPrototypeInner> {
        self.inner.lock().unwrap()
    }

    pub fn inner(&self) -> &Arc<Mutex<FrameworkEntityPrototypeInner>> {
        &self.inner
    }
}

impl EntityPrototype for FrameworkEntityPrototype {
    fn create(&self, resource_map: &ResourceMap, entity: &EntityRef) {
        let entity_prototype_guard = self.lock();
        for c in entity_prototype_guard.component_prototypes() {
            c.enqueue_create(resource_map, &entity.handle());
        }

        #[cfg(feature = "editor")]
        {
            let mut selection_shapes = vec![];

            for c in entity_prototype_guard.component_prototypes() {
                let select_registry = resource_map.fetch::<SelectRegistry>();
                if let Some(shape) = select_registry.create_selection_shape(&**c) {
                    selection_shapes.push(shape);
                }
            }

            // If we detect any components that want to be selectable, attach an EditorShapeComponentPrototype
            // to the entity with those shapes
            if !selection_shapes.is_empty() {
                let compound_shape = ncollide2d::shape::Compound::new(selection_shapes);
                let compound_shape_handle = ncollide2d::shape::ShapeHandle::new(compound_shape);
                let editor_shape_component_prototype =
                    EditorShapeComponentPrototype::new(compound_shape_handle);

                editor_shape_component_prototype.enqueue_create(resource_map, &entity.handle());
            }
        }

        // if the entity is persistent, attach a PersistentEntityComponent to it
        match self.persistence_policy {
            FrameworkEntityPersistencePolicy::Persistent => {
                // Add PersistentEntityComponent to any component that is persistent
                let mut storage =
                    resource_map.fetch_mut::<<PersistentEntityComponent as Component>::Storage>();
                entity.add_component(&mut *storage, PersistentEntityComponent::new(self.clone())).unwrap();
            }
            _ => {}
        }
    }
}
