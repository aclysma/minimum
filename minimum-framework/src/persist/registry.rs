#[cfg(feature = "editor")]
use crate::components::PersistentEntityComponent;

use crate::persist::ComponentPrototypeSerializer;
use crate::{
    FrameworkComponentPrototypeDyn, FrameworkEntityPersistencePolicy, FrameworkEntityPrototype, FrameworkComponentPrototype
};
use hashbrown::HashMap;
use minimum::EntityPrototype;
use minimum::ResourceMap;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct SavedComponent {
    pub type_name: String,
    pub data: serde_json::Value,
}

impl SavedComponent {
    #[cfg(feature = "editor")]
    pub fn new(type_name: String, data: serde_json::Value) -> Self {
        SavedComponent { type_name, data }
    }
}

#[derive(Serialize, Deserialize)]
struct SavedObject {
    pub saved_components: Vec<SavedComponent>,
}

impl SavedObject {
    #[cfg(feature = "editor")]
    pub fn new(saved_components: Vec<SavedComponent>) -> Self {
        SavedObject { saved_components }
    }
}

#[derive(Serialize, Deserialize)]
struct LevelFile {
    pub saved_objects: Vec<SavedObject>,
}

impl LevelFile {
    #[cfg(feature = "editor")]
    pub fn new(saved_objects: Vec<SavedObject>) -> Self {
        LevelFile { saved_objects }
    }
}

pub struct ComponentPrototypeMetadata {
    component_type_id: std::any::TypeId,
    component_prototype_type_id: std::any::TypeId,
    component_prototype_type_name: &'static str
}

impl ComponentPrototypeMetadata {
    fn of_type<T>(name: &'static str) -> Self
        where T: FrameworkComponentPrototypeDyn + FrameworkComponentPrototype
    {
        ComponentPrototypeMetadata {
            component_type_id: <T as FrameworkComponentPrototype>::component_type(),
            component_prototype_type_id: std::any::TypeId::of::<T>(),
            component_prototype_type_name: name
        }
    }

    pub fn name(&self) -> &'static str {
        self.component_prototype_type_name
    }

    pub fn type_id(&self) -> &std::any::TypeId {
        &self.component_prototype_type_id
    }

    pub fn component_type_id(&self) -> &std::any::TypeId {
        &self.component_type_id
    }
}

trait RegisteredComponentPrototypeTrait: Send + Sync {
    fn serialize(
        &self,
        component_prototype: &dyn FrameworkComponentPrototypeDyn,
    ) -> Result<serde_json::Value, failure::Error>;

    fn deserialize(
        &self,
        data: serde_json::Value,
    ) -> Result<Box<dyn FrameworkComponentPrototypeDyn>, failure::Error>;

    fn create_default(&self) -> Box<dyn FrameworkComponentPrototypeDyn>;

    fn component_type(&self) -> std::any::TypeId;

    fn metadata(&self) -> &ComponentPrototypeMetadata;
}

struct RegisteredComponentPrototype<T> {
    phantom_data: PhantomData<T>,
    metadata: ComponentPrototypeMetadata,
}

impl<T> RegisteredComponentPrototype<T>
where T: FrameworkComponentPrototypeDyn + FrameworkComponentPrototype {
    fn new(name: &'static str) -> Self {
        RegisteredComponentPrototype {
            phantom_data: PhantomData,
            metadata: ComponentPrototypeMetadata::of_type::<T>(name),
        }
    }
}

impl<T> RegisteredComponentPrototypeTrait for RegisteredComponentPrototype<T>
where
    T: FrameworkComponentPrototypeDyn + FrameworkComponentPrototype + Default + ComponentPrototypeSerializer<T>,
{
    fn serialize(
        &self,
        component_prototype: &dyn FrameworkComponentPrototypeDyn,
    ) -> Result<serde_json::Value, failure::Error> {
        let t = component_prototype.downcast_ref::<T>().unwrap();

        <T as ComponentPrototypeSerializer<T>>::serialize(t)
    }

    fn deserialize(
        &self,
        data: serde_json::Value,
    ) -> Result<Box<dyn FrameworkComponentPrototypeDyn>, failure::Error> {
        Ok(Box::new(
            <T as ComponentPrototypeSerializer<T>>::deserialize(data)?,
        ))
    }

    fn create_default(&self) -> Box<dyn FrameworkComponentPrototypeDyn> {
        Box::new(T::default())
    }

    fn component_type(&self) -> std::any::TypeId {
        <T as FrameworkComponentPrototype>::component_type()
    }

    fn metadata(&self) -> &ComponentPrototypeMetadata {
        &self.metadata
    }
}

//
// Serialization Error
//
#[derive(Debug, failure::Fail)]
pub enum SerializeError {
    #[fail(display = "Serde Error: {:?}", 0)]
    SerdeError(serde_json::error::Error),
    #[fail(display = "IO Error: {:?}", 0)]
    IoError(std::io::Error),
}

impl From<serde_json::error::Error> for SerializeError {
    fn from(inner: serde_json::error::Error) -> SerializeError {
        SerializeError::SerdeError(inner)
    }
}

impl From<std::io::Error> for SerializeError {
    fn from(inner: std::io::Error) -> SerializeError {
        SerializeError::IoError(inner)
    }
}

#[derive(Debug, failure::Fail)]
pub enum DeserializeError {
    #[fail(display = "Unregistered Type: {:?}", 0)]
    UnregisteredType(String),
    #[fail(display = "Serde Error: {:?}", 0)]
    SerdeError(serde_json::error::Error),
    #[fail(display = "IO Error: {:?}", 0)]
    IoError(std::io::Error),
}

impl From<serde_json::error::Error> for DeserializeError {
    fn from(inner: serde_json::error::Error) -> DeserializeError {
        DeserializeError::SerdeError(inner)
    }
}

impl From<std::io::Error> for DeserializeError {
    fn from(inner: std::io::Error) -> DeserializeError {
        DeserializeError::IoError(inner)
    }
}

//
// ComponentRegistry
//
pub struct PersistRegistry {
    registered_component_prototypes_by_type_id:
        HashMap<std::any::TypeId, Arc<dyn RegisteredComponentPrototypeTrait>>,

    //TODO: Shipped code would want to do this by hash instead of string. We can still save the string
    // in an Option<String> strictly for debug purposes, but we shouldn't rely on it.
    registered_component_prototypes_by_string_name:
        HashMap<String, Arc<dyn RegisteredComponentPrototypeTrait>>,
}

impl PersistRegistry {
    pub fn new() -> Self {
        PersistRegistry {
            registered_component_prototypes_by_type_id: HashMap::new(),
            registered_component_prototypes_by_string_name: HashMap::new(),
        }
    }

    pub fn register_component_prototype<
        T: FrameworkComponentPrototypeDyn + FrameworkComponentPrototype + ComponentPrototypeSerializer<T> + Default,
    >(
        &mut self,
        name: &'static str,
    ) {
        println!("register component prototype {}: {:?}", name, std::any::TypeId::of::<T>());
        self.registered_component_prototypes_by_type_id.insert(
            std::any::TypeId::of::<T>(),
            Arc::new(RegisteredComponentPrototype::<T>::new(name)),
        );

        self.registered_component_prototypes_by_string_name.insert(
            name.to_string(),
            Arc::new(RegisteredComponentPrototype::<T>::new(name)),
        );
    }

    pub fn load<P: AsRef<std::path::Path>>(
        &self,
        resource_map: &ResourceMap,
        path: P,
    ) -> Result<(), DeserializeError> {
        let input = std::fs::read_to_string(path)?;
        let serialized_level = serde_json::from_str::<LevelFile>(&input)?;

        let mut entities: Vec<Box<dyn EntityPrototype>> = vec![];
        for entity in serialized_level.saved_objects {
            let mut deserialized_components: Vec<Box<dyn FrameworkComponentPrototypeDyn>> = vec![];
            for component in entity.saved_components {
                // Resolve the registered component that is able to serialize this component
                let registered_type = self
                    .registered_component_prototypes_by_string_name
                    .get(&component.type_name);

                // Try to serialize the component
                match registered_type {
                    Some(registered_type) => {
                        match (*registered_type).deserialize(component.data) {
                            Ok(deserialized_component) => {
                                deserialized_components.push(deserialized_component);
                            },
                            Err(e) => {
                                println!("Failed to load component: {}", e);
                            }
                        }
                    }
                    None => {
                        // Skip unknown types
                        return Err(DeserializeError::UnregisteredType(component.type_name));
                    }
                }
            }

            entities.push(Box::new(FrameworkEntityPrototype::new(
                std::path::PathBuf::from("loaded from file"),
                FrameworkEntityPersistencePolicy::Persistent,
                deserialized_components,
            )));
        }

        let mut entity_factory = resource_map.fetch_mut::<minimum::EntityFactory>();
        for e in entities {
            entity_factory.enqueue_create(e);
        }

        Ok(())
    }

    #[cfg(feature = "editor")]
    pub fn save<P: AsRef<std::path::Path>>(
        &self,
        resource_map: &ResourceMap,
        path: P,
    ) -> Result<(), SerializeError> {
        use minimum::Component;
        use minimum::EntitySet;

        // Get the entities and all persistent entity components. This represents all the data we need to save
        let entity_set = resource_map.fetch::<EntitySet>();
        let persistent_entity_components =
            resource_map.fetch::<<PersistentEntityComponent as Component>::Storage>();

        // Iterate entities and their entity prototypes, adding them to saved_objects
        let mut saved_objects = vec![];
        for (_entity_handle, component_prototype) in persistent_entity_components.iter(&*entity_set)
        {
            // Access the data in the prototype.
            let arc = component_prototype.entity_prototype().inner().clone();
            let pep = arc.lock().unwrap();

            // Iterate their component prototypes, adding them to the saved_components list
            let mut saved_components = vec![];
            for component_prototype in pep.component_prototypes() {
                // Resolve the registered component that is able to serialize this component
                let component_prototype_type =
                    FrameworkComponentPrototypeDyn::type_id(&**component_prototype);
                let registered_type = self
                    .registered_component_prototypes_by_type_id
                    .get(&component_prototype_type);

                // Try to serialize the component
                match registered_type {
                    Some(registered_type) => {
                        let serialized_data = (*registered_type)
                            .serialize(&**component_prototype)
                            .unwrap();
                        saved_components.push(SavedComponent::new(
                            registered_type.metadata().name().to_string(),
                            serialized_data,
                        ));
                    }
                    None => {
                        // Skip unknown types
                        //return Err(SerializeError::UnregisteredType(component_prototype_type))
                    }
                }
            }

            saved_objects.push(SavedObject::new(saved_components));
        }

        let serialized_level = LevelFile::new(saved_objects);
        let str = serde_json::to_string_pretty(&serialized_level)?;

        std::fs::write(path, str)?;
        Ok(())
    }

    pub fn iter_metadata(&self) -> impl Iterator<Item = &'_ ComponentPrototypeMetadata> + '_ {
        self.registered_component_prototypes_by_type_id
            .iter()
            .map(|(_, x)| (x.metadata()))
    }

    pub fn create_default(
        &self,
        type_id: std::any::TypeId,
    ) -> Box<dyn FrameworkComponentPrototypeDyn> {
        self.registered_component_prototypes_by_type_id[&type_id].create_default()
    }
}
