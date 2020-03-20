use legion::prelude::*;
use legion::world::{NoneCloneImplResult, NoneEntityReplacePolicy};
use glam::Vec2;

use type_uuid::TypeUuid;

use serde::Serialize;
use serde::Deserialize;
use serde_diff::SerdeDiff;
use std::collections::HashMap;
use legion_prefab::SpawnCloneImpl;

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

#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107190002"]
pub struct AccelerationComponent {
    #[serde_diff(opaque)]
    pub value: Vec2,
}

legion_prefab::register_component_type!(AccelerationComponent);

struct Gravity(pub glam::Vec2);

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

    let component_registry = minimum::ComponentRegistryBuilder::new()
        .auto_register_components()
        .build();

    // Create a legion universe
    let universe = Universe::new();

    // Create a world and insert data into it that we would like to save into a prefab
    let mut prefab_world = universe.create_world();
    let prefab_entity = *prefab_world.insert(
        (),
        (0..1).map(|_| (PositionComponent { value: Vec2::new(0.0, 500.0)}, VelocityComponent { value: Vec2::new(5.0, 0.0) }))
    ).first().unwrap();

    let prefab = legion_prefab::Prefab::new(prefab_world);

    let world_as_string = ron::ser::to_string_pretty(&prefab, ron::ser::PrettyConfig::default()).unwrap();
    println!("Save world: {}", world_as_string);

    let prefab : legion_prefab::Prefab = ron::de::from_str(&world_as_string).unwrap();

    let prefab_cook_order = vec![prefab.prefab_id()];
    let mut prefab_lookup = HashMap::new();
    prefab_lookup.insert(prefab.prefab_id(), &prefab);

    let cooked = legion_prefab::cook_prefab(
        &universe,
        component_registry.components(),
        component_registry.components_by_uuid(),
        prefab_cook_order.as_slice(),
        &prefab_lookup
    );

    let resources = Resources::default();
    let spawn_impl = component_registry.spawn_clone_impl(&resources);

    let mut world = universe.create_world();
    let mut clone_impl_result = HashMap::new();
    world.clone_from(&cooked.world, &spawn_impl, &mut legion::world::HashMapCloneImplResult(&mut clone_impl_result), &legion::world::NoneEntityReplacePolicy);

    let mut resources = Resources::default();

    // Insert a resource that can be queried to find gravity
    resources.insert(Gravity(-9.8 * Vec2::unit_y()));

    // Insert an object with position and velocity

    for _ in 0..10 {
        // Fetch gravity... and integrate it to velocity.
        let gravity = resources.get::<Gravity>().unwrap();
        let query = <(Write<VelocityComponent>)>::query();
        for mut vel in query.iter_mut(&mut world) {
            vel.value += gravity.0;
        }

        // Iterate across all entities and integrate velocity to position
        let query = <(Write<PositionComponent>, TryRead<VelocityComponent>)>::query();
        for (mut pos, vel) in query.iter_mut(&mut world) {
            if let Some(vel) = vel {
                pos.value += vel.value;
            }

            pos.value += gravity.0;
        }

        let world_entity = clone_impl_result[&prefab_entity];
        let position = world.get_component::<PositionComponent>(world_entity).unwrap();
        println!("Position is {}", position.value);
    }
}
