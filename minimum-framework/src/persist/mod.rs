mod registry;

pub use registry::PersistRegistry;
use serde::de::DeserializeOwned;
pub use serde::{Deserialize, Serialize};

/// Used for serialization of component prototypes
pub trait ComponentPrototypeSerializer<T>: Send + Sync {
    fn serialize(prototype: &T) -> Result<serde_json::Value, failure::Error>;
    fn deserialize(data: serde_json::Value) -> Result<T, failure::Error>;
}

impl<T> ComponentPrototypeSerializer<T> for T
where
    T: Serialize + DeserializeOwned + Sync + Send,
{
    fn serialize(prototype: &T) -> Result<serde_json::Value, failure::Error> {
        Ok(serde_json::to_value(prototype)?)
    }

    fn deserialize(data: serde_json::Value) -> Result<T, failure::Error> {
        Ok(serde_json::from_value(data)?)
    }
}
