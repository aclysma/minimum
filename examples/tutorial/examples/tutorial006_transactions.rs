use legion::*;

use glam::Vec2;

use type_uuid::TypeUuid;

use serde::Serialize;
use serde::Deserialize;
use serde_diff::SerdeDiff;
use std::collections::HashMap;
use legion_prefab::{SpawnCloneImpl, Prefab, ComponentRegistration, CookedPrefab};

use minimum::ComponentRegistry;
use prefab_format::EntityUuid;
use legion_transaction::{TransactionDiffs};

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
    // Cook the prefab world we just deserialized
    //
    let prefab_cook_order = vec![prefab.prefab_id()];
    let mut prefab_lookup = HashMap::new();
    prefab_lookup.insert(prefab.prefab_id(), &prefab);

    let cooked_prefab = legion_prefab::cook_prefab(
        component_registry.components(),
        component_registry.components_by_uuid(),
        prefab_cook_order.as_slice(),
        &prefab_lookup,
    );

    // Look up the entity associated with the entity_uuid
    let cooked_entity = cooked_prefab.entities[&entity_uuid];

    // Start a new transaction
    let mut transaction = legion_transaction::TransactionBuilder::new()
        .add_entity(cooked_entity, entity_uuid)
        .begin(&cooked_prefab.world, &component_registry.copy_clone_impl());

    // Mess with a value in the transaction's world
    let transaction_entity = transaction.uuid_to_entity(entity_uuid).unwrap();
    transaction
        .world_mut()
        .get_component_mut::<PositionComponent>(transaction_entity)
        .unwrap()
        .value += glam::Vec2::new(0.0, 1000.0);

    // Produce diffs based on the edit
    let diffs = transaction.create_transaction_diffs(component_registry.components_by_uuid());

    // Show how this is used with uncooked prefabs
    apply_to_prefab(prefab, &component_registry, entity_uuid, &diffs);

    // Show how this is used with cooked prefabs
    apply_to_cooked_prefab(cooked_prefab, &component_registry, entity_uuid, &diffs);
}

fn apply_to_prefab(
    mut prefab: Prefab,
    component_registry: &ComponentRegistry,
    entity_uuid: EntityUuid,
    diffs: &TransactionDiffs,
) {
    println!(
        "Original value on prefab: {}",
        prefab
            .world
            .get_component::<PositionComponent>(prefab.prefab_meta.entities[&entity_uuid])
            .unwrap()
            .value
    );

    // Apply the change to the prefab
    // The return value is a result that may indicate failure if there are prefab overrides
    let mut prefab = legion_transaction::apply_diff_to_prefab(
        &mut prefab,
        diffs.apply_diff(),
        component_registry.components_by_uuid(),
        &component_registry.copy_clone_impl(),
    )
    .unwrap();

    println!(
        "Modified value on prefab: {}",
        prefab
            .world
            .get_component::<PositionComponent>(prefab.prefab_meta.entities[&entity_uuid])
            .unwrap()
            .value
    );

    // Revert the change
    // The return value is a result that may indicate failure if there are prefab overrides
    let prefab = legion_transaction::apply_diff_to_prefab(
        &mut prefab,
        diffs.revert_diff(),
        component_registry.components_by_uuid(),
        &component_registry.copy_clone_impl(),
    )
    .unwrap();

    println!(
        "Revert value on prefab: {}",
        prefab
            .world
            .get_component::<PositionComponent>(prefab.prefab_meta.entities[&entity_uuid])
            .unwrap()
            .value
    );
}

fn apply_to_cooked_prefab(
    mut cooked_prefab: CookedPrefab,
    component_registry: &ComponentRegistry,
    entity_uuid: EntityUuid,
    diffs: &TransactionDiffs,
) {
    println!(
        "Original value on cooked prefab: {}",
        cooked_prefab
            .world
            .get_component::<PositionComponent>(cooked_prefab.entities[&entity_uuid])
            .unwrap()
            .value
    );

    // Apply the change to the prefab
    let mut cooked_prefab = legion_transaction::apply_diff_to_cooked_prefab(
        &mut cooked_prefab,
        diffs.apply_diff(),
        component_registry.components_by_uuid(),
        &component_registry.copy_clone_impl(),
    );

    println!(
        "Modified value on cooked prefab: {}",
        cooked_prefab
            .world
            .get_component::<PositionComponent>(cooked_prefab.entities[&entity_uuid])
            .unwrap()
            .value
    );

    // Revert the change
    let cooked_prefab = legion_transaction::apply_diff_to_cooked_prefab(
        &mut cooked_prefab,
        diffs.revert_diff(),
        component_registry.components_by_uuid(),
        &component_registry.copy_clone_impl(),
    );

    println!(
        "Revert value on cooked prefab: {}",
        cooked_prefab
            .world
            .get_component::<PositionComponent>(cooked_prefab.entities[&entity_uuid])
            .unwrap()
            .value
    );
}
