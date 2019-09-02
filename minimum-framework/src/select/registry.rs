
use crate::FrameworkComponentPrototype;
use hashbrown::HashMap;
use std::marker::PhantomData;
use super::SelectableComponentPrototype;

trait RegisteredComponentPrototypeTrait: Send + Sync {
    fn create_selection_shape(
        &self,
        component_prototype: &dyn FrameworkComponentPrototype
    )-> (ncollide2d::math::Isometry<f32>, ncollide2d::shape::ShapeHandle<f32>);
}

struct RegisteredComponentPrototype<T> {
    phantom_data: PhantomData<T>
}

impl<T> RegisteredComponentPrototype<T> {
    fn new() -> Self {
        RegisteredComponentPrototype {
            phantom_data: PhantomData
        }
    }
}

impl<T> RegisteredComponentPrototypeTrait for RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototype + SelectableComponentPrototype<T>,
{
    fn create_selection_shape(
        &self,
        component_prototype: &dyn FrameworkComponentPrototype
    ) -> (ncollide2d::math::Isometry<f32>, ncollide2d::shape::ShapeHandle<f32>) {
        let t = component_prototype.downcast_ref::<T>().unwrap();
        <T as SelectableComponentPrototype<T>>::create_selection_shape(t)
    }
}

//
// ComponentRegistry
//
pub struct SelectRegistry {
    registered_component_prototypes:
        HashMap<std::any::TypeId, Box<dyn RegisteredComponentPrototypeTrait>>,
}

impl SelectRegistry {
    pub fn new() -> Self {
        SelectRegistry {
            registered_component_prototypes: HashMap::new(),
        }
    }

    pub fn register_component_prototype<
        T: FrameworkComponentPrototype + SelectableComponentPrototype<T>,
    >(
        &mut self,
    ) {
        self.registered_component_prototypes.insert(
            std::any::TypeId::of::<T>(),
            Box::new(RegisteredComponentPrototype::<T>::new()),
        );
    }

    pub fn create_selection_shape(&self, component_prototype: &dyn FrameworkComponentPrototype) -> Option<(ncollide2d::math::Isometry<f32>, ncollide2d::shape::ShapeHandle<f32>)> {
        let component_prototype_type = FrameworkComponentPrototype::type_id(component_prototype);

        if let Some(registered) = self.registered_component_prototypes.get(&component_prototype_type) {
            Some(registered.create_selection_shape(component_prototype))
        } else {
            None
        }
    }
}
