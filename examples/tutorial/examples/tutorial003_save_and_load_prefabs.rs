use legion::*;

use glam::Vec2;

use type_uuid::TypeUuid;

use serde::Serialize;
use serde::Deserialize;
use serde_diff::SerdeDiff;

use legion_prefab::{SpawnCloneImpl, Prefab, ComponentRegistration};

use minimum::ComponentRegistry;

#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107190000"]
pub struct PositionComponent {
    #[serde_diff(opaque)]
    pub value: Vec2,
}

legion_prefab::register_component_type!(PositionComponent);

#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107190001"]
pub struct VelocityComponent {
    #[serde_diff(opaque)]
    pub value: Vec2,
}

legion_prefab::register_component_type!(VelocityComponent);

fn main() {
    // Setup logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // Spawn the daemon in a background thread. This could be a different process, but
    // for simplicity we'll launch it here.
    std::thread::spawn(move || {
        minimum::daemon::run();
    });

    // Register all components (based on legion_prefab::register_component_type! macro)
    let component_registry = minimum::ComponentRegistryBuilder::new()
        .auto_register_components()
        .build();

    // Create a world and insert data into it that we would like to save into a prefab
    let mut prefab_world = World::default();
    let prefab_entity = *prefab_world
        .insert(
            (),
            (0..1).map(|_| {
                (
                    PositionComponent {
                        value: Vec2::new(0.0, 500.0),
                    },
                    VelocityComponent {
                        value: Vec2::new(5.0, 0.0),
                    },
                )
            }),
        )
        .first()
        .unwrap();

    // Create the prefab
    let prefab_uncooked = Prefab::new(prefab_world);

    // Get the UUID of the entity. This UUID is maintained throughout the whole chain.
    let entity_uuid = prefab_uncooked
        .prefab_meta
        .entities
        .iter()
        .find(|(_, value)| **value == prefab_entity)
        .map(|(entity_uuid, _)| *entity_uuid)
        .unwrap();

    // Serialize the prefab to a string
    let serialized_prefab = serialize_prefab(&component_registry, &prefab_uncooked);

    println!("Prefab serialized to string");
    println!("{}", serialized_prefab);

    // Deserialize the world from a string
    let deserialized_prefab = deserialize_prefab(&component_registry, &serialized_prefab);

    // Look up the entity associated with the entity_uuid. To do this, we have to:
    // - Look at the original prefab to find the UUID of the entity
    // - Then use prefab_meta on the deserialized prefab to find the entity in the deserialized
    //   prefab's world
    let deserialized_entity = deserialized_prefab.prefab_meta.entities[&entity_uuid];

    // Now run some code to demonstrate that we found the same entity in the deserialized world and
    // that we get the same results as before
    let position = deserialized_prefab
        .world
        .get_component::<PositionComponent>(deserialized_entity)
        .unwrap();

    println!(
        "Position of {} is {}",
        uuid::Uuid::from_bytes(entity_uuid),
        position.value
    );
}

fn serialize_prefab(
    component_registry: &ComponentRegistry,
    prefab: &Prefab,
) -> String {
    let prefab_serde_context = legion_prefab::PrefabSerdeContext {
        registered_components: component_registry.components_by_uuid(),
    };

    let mut ron_ser = ron::ser::Serializer::new(Some(ron::ser::PrettyConfig::default()), true);
    let prefab_ser = legion_prefab::PrefabFormatSerializer::new(prefab_serde_context, prefab);

    prefab_format::serialize(&mut ron_ser, &prefab_ser, prefab.prefab_id())
        .expect("failed to round-trip prefab");

    ron_ser.into_output_string()
}

fn deserialize_prefab(
    component_registry: &ComponentRegistry,
    serialized: &str,
) -> Prefab {
    let prefab_serde_context = legion_prefab::PrefabSerdeContext {
        registered_components: component_registry.components_by_uuid(),
    };

    let mut deserializer = ron::de::Deserializer::from_str(serialized).unwrap();

    let prefab_deser = legion_prefab::PrefabFormatDeserializer::new(prefab_serde_context);
    prefab_format::deserialize(&mut deserializer, &prefab_deser).unwrap();
    prefab_deser.prefab()
}
