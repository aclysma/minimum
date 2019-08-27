use minimum::entity::EntityPrototype;
use minimum::Component;

use minimum::EntityRef;
use minimum::ResourceMap;

use std::sync::Arc;
use std::sync::Mutex;

use crate::components::PersistentEntityComponent;

// impl ComponentPrototype for FrameworkComponentPrototype?
pub trait FrameworkComponentPrototype:
    minimum::component::ComponentCreator + named_type::NamedType + mopa::Any
{
}

//TODO: Get rid of NamedType
/*
/// A trait for getting the name of a type
pub trait NamedType {
    /// Returns the canonical name with the fully qualified module name for the
    /// given type
    fn type_name() -> &'static str
        where
            Self: Sized;
}

impl<T : FrameworkComponentPrototype> named_type::NamedType for T {
    fn type_name() -> &'static str {
        core::any::type_name::<T>()
    }
}
*/

mopafy!(FrameworkComponentPrototype);

pub struct FrameworkEntityPrototypeInner {
    path: std::path::PathBuf,
    component_prototypes: Vec<Box<dyn FrameworkComponentPrototype>>,
}

impl FrameworkEntityPrototypeInner {
    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }

    pub fn component_prototypes(&self) -> &Vec<Box<dyn FrameworkComponentPrototype>> {
        &self.component_prototypes
    }

    pub fn component_prototypes_mut<'a: 'b, 'b>(
        &'a mut self,
    ) -> &'b mut Vec<Box<dyn FrameworkComponentPrototype>> {
        &mut self.component_prototypes
    }
}

#[derive(Clone)]
pub struct FrameworkEntityPrototype {
    inner: Arc<Mutex<FrameworkEntityPrototypeInner>>,
}

impl FrameworkEntityPrototype {
    pub fn new(
        path: std::path::PathBuf,
        component_prototypes: Vec<Box<dyn FrameworkComponentPrototype>>,
    ) -> Self {
        FrameworkEntityPrototype {
            inner: Arc::new(Mutex::new(FrameworkEntityPrototypeInner {
                path,
                component_prototypes,
            })),
        }
    }

    pub fn get_mut(&self) -> std::sync::MutexGuard<FrameworkEntityPrototypeInner> {
        self.inner.lock().unwrap()
    }

    pub fn inner(&self) -> &Arc<Mutex<FrameworkEntityPrototypeInner>> {
        &self.inner
    }
}

impl EntityPrototype for FrameworkEntityPrototype {
    fn create(&self, resource_map: &ResourceMap, entity: &EntityRef) {
        let p = self.get_mut();
        for c in p.component_prototypes() {
            c.enqueue_create(resource_map, &entity.handle());
        }

        let mut storage =
            resource_map.fetch_mut::<<PersistentEntityComponent as Component>::Storage>();
        entity.add_component(&mut *storage, PersistentEntityComponent::new(self.clone()));
    }
}
