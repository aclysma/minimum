use crate::components::PersistentEntityComponent;
use crate::framework::persist::ComponentPrototypeSerializer;
use crate::framework::FrameworkComponentPrototype;
use hashbrown::HashMap;
use minimum::Component;
use minimum::EntitySet;
use minimum::ResourceMap;
use std::marker::PhantomData;

trait RegisteredComponentPrototypeTrait: Send + Sync {
    fn serialize(
        &self,
        component_prototype: &dyn FrameworkComponentPrototype,
    ) -> Result<String, failure::Error>;

    fn deserialize(
        &self,
        data: &str,
    ) -> Result<Box<dyn FrameworkComponentPrototype>, failure::Error>;

    fn default(&self) -> Box<dyn FrameworkComponentPrototype>;

    fn name(&self) -> &'static str;
}

struct RegisteredComponentPrototype<T> {
    phantom_data: PhantomData<T>,
    name: &'static str
}

impl<T> RegisteredComponentPrototype<T> {
    fn new(name: &'static str) -> Self {
        RegisteredComponentPrototype {
            phantom_data: PhantomData,
            name
        }
    }
}

impl<T> RegisteredComponentPrototypeTrait for RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototype + Default + ComponentPrototypeSerializer<T>,
{
    fn serialize(
        &self,
        component_prototype: &dyn FrameworkComponentPrototype,
    ) -> Result<String, failure::Error> {
        let t = component_prototype.downcast_ref::<T>().unwrap();

        <T as ComponentPrototypeSerializer<T>>::serialize(t)
    }

    fn deserialize(
        &self,
        data: &str,
    ) -> Result<Box<dyn FrameworkComponentPrototype>, failure::Error> {
        Ok(Box::new(
            <T as ComponentPrototypeSerializer<T>>::deserialize(data)?,
        ))
    }

    fn default(&self) -> Box<dyn FrameworkComponentPrototype> {
        Box::new(T::default())
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

//
// ComponentRegistry
//
pub struct PersistRegistry {
    registered_component_prototypes:
        HashMap<std::any::TypeId, Box<dyn RegisteredComponentPrototypeTrait>>,
}

impl PersistRegistry {
    pub fn new() -> Self {
        PersistRegistry {
            registered_component_prototypes: HashMap::new(),
        }
    }

    pub fn register_component_prototype<
        T: FrameworkComponentPrototype + ComponentPrototypeSerializer<T> + Default,
    >(
        &mut self,
        name: &'static str
    ) {
        self.registered_component_prototypes.insert(
            std::any::TypeId::of::<T>(),
            Box::new(RegisteredComponentPrototype::<T>::new(name)),
        );
    }

    pub fn save(&self, resource_map: &ResourceMap) {
        let entity_set = resource_map.fetch::<EntitySet>();
        let persistent_entity_components =
            resource_map.fetch::<<PersistentEntityComponent as Component>::Storage>();

        // Iterate entities and their entity prototypes
        for (entity_handle, component_prototype) in persistent_entity_components.iter(&*entity_set)
        {
            let arc = component_prototype.entity_prototype().inner().clone();
            let pep = arc.lock().unwrap();

            // Iterate their component prototypes
            for component_prototype in pep.component_prototypes() {
                let component_prototype_type =
                    FrameworkComponentPrototype::type_id(&**component_prototype);

                // Try to save each component prototype
                println!("{:?} {:?}", entity_handle, component_prototype_type);
                let registered = self
                    .registered_component_prototypes
                    .get(&component_prototype_type);
                match registered {
                    Some(r) => {
                        //r.serialize(&**component_prototype);
                        let s = (*r).serialize(&**component_prototype).unwrap();
                        //r.deserialize(&s);
                        //let loaded = r.
                        println!("SAVED DATA: {}", s);
                    }
                    None => {} //panic!("Unregistered component prototype cannot be persisted")
                }
            }
        }
    }

    pub fn iter_names(&self) -> impl Iterator<Item=(&'_ std::any::TypeId, &'static str)> + '_ {
        self.registered_component_prototypes.iter().map(|(t, x)| (t, x.name()))
    }

    pub fn create_default(&self, type_id: &std::any::TypeId) -> Box<dyn FrameworkComponentPrototype> {
        self.registered_component_prototypes[type_id].default()
    }
}
