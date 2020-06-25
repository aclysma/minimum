use legion::prelude::*;

use glam::Vec2;

use type_uuid::TypeUuid;

use serde::Serialize;
use serde::Deserialize;
use serde_diff::SerdeDiff;
use std::collections::HashMap;
use legion_prefab::{SpawnCloneImpl, Prefab, ComponentRegistration};

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

    // Create a legion universe
    let universe = Universe::new();

    // Create a world and insert data into it that we would like to save into a prefab
    let mut prefab_world = universe.create_world();
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
    let prefab = Prefab::new(prefab_world);

    // Get the UUID of the entity. This UUID is maintained throughout the whole chain.
    let entity_uuid = prefab
        .prefab_meta
        .entities
        .iter()
        .find(|(_, value)| **value == prefab_entity)
        .map(|(entity_uuid, _)| *entity_uuid)
        .unwrap();

    //
    // Cook the prefab world
    //

    // Prefabs must be cooked in dependency order, based on overrides. For example, if prefab B
    // overrides something on prefab A, and prefab C overrides something on prefab B, we must cook
    // A, then B, then C. When cooking B, we must be able to access A in cooked form, and when
    // cooking C, we must be able to access B in cooked form.
    //
    // In this case there are no overrides, so our ordering is simply the prefab we want to cook
    // and the lookup just contains this prefab.
    //
    // In the future this can happen in atelier as part of the asset build process. For now,
    // cooking happens at runtime in minimum but dependency management is automatically handled.
    let prefab_cook_order = vec![prefab.prefab_id()];
    let mut prefab_lookup = HashMap::new();
    prefab_lookup.insert(prefab.prefab_id(), &prefab);

    let cooked_prefab = legion_prefab::cook_prefab(
        &universe,
        component_registry.components(),
        component_registry.components_by_uuid(),
        prefab_cook_order.as_slice(),
        &prefab_lookup,
    );

    // Look up the entity associated with the entity_uuid
    let cooked_entity = cooked_prefab.entities[&entity_uuid];

    let position = cooked_prefab
        .world
        .get_component::<PositionComponent>(cooked_entity)
        .unwrap();
    println!(
        "Position of {} is {}",
        uuid::Uuid::from_bytes(entity_uuid),
        position.value
    );
}
