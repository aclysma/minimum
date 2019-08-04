use minimum::resource::{DataRequirement, Read, ResourceMap, ResourceMapBuilder, Write};

use minimum::resource::async_dispatch::Task;

use minimum::async_dispatcher::ExecuteSequential;

use minimum::component::{Component, ComponentStorage};
use minimum::{EntitySet, ReadComponent, WriteComponent};

mod shared;

use shared::components::{PositionComponent, SpeedMultiplierComponent, VelocityComponent};

use shared::resources::{TimeState, UpdateCount};

use shared::Vec2;

struct UpdatePositions;
impl Task for UpdatePositions {
    type RequiredResources = (
        Read<TimeState>,
        Read<EntitySet>,
        ReadComponent<VelocityComponent>,
        WriteComponent<PositionComponent>,
        ReadComponent<SpeedMultiplierComponent>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            time_state,
            game_entities,
            velocity_components,
            mut position_components,
            speed_multiplier_components,
        ) = data;

        // EXAMPLE: iterate entity handles
        println!("entity count {:?}", game_entities.iter().count());
        for entity in game_entities.iter() {
            println!("all entities: {:?}", entity.handle());
        }

        // EXAMPLE: non-mutable iterate over entities with velocity components
        for (entity_handle, vel_component) in velocity_components.iter(&game_entities) {
            println!(
                "entities with velocity: E: {:?} V: {:?}",
                entity_handle, vel_component
            );
        }

        //EXAMPLE: mutable iterate over entities with position components
        for (entity_handle, pos_component) in position_components.iter_mut(&game_entities) {
            pos_component.position.y += 10.0;
            println!(
                "entities with position: E: {:?} P: {:?}",
                entity_handle, pos_component
            );
        }

        //EXAMPLE: iterate over entities:
        // - mutable position
        // - immutable velocity (use get_mut for mutable component
        // - optional speed multiplier
        for (entity_index, pos) in position_components.iter_mut(&game_entities) {
            if let (Some(vel), mul) = (
                velocity_components.get(&entity_index),
                speed_multiplier_components.get(&entity_index),
            ) {
                println!("p {:?} v {:?} m {:?}", pos, vel, mul);
                let multiplier = time_state.dt * mul.map(|x| x.multiplier).unwrap_or(1.0);
                pos.position.x += multiplier * vel.velocity.x;
                pos.position.y += multiplier * vel.velocity.y;
            }
        }
    }
}

//TODO: Rewrite to use an entity prototype
fn create_objects(resource_map: &ResourceMap) {
    let mut game_entities = resource_map.fetch_mut::<EntitySet>();
    let mut pos_components = resource_map.fetch_mut::<<PositionComponent as Component>::Storage>();
    let mut vel_components = resource_map.fetch_mut::<<VelocityComponent as Component>::Storage>();
    let mut speed_multiplier_components =
        resource_map.fetch_mut::<<SpeedMultiplierComponent as Component>::Storage>();

    for i in 0..10 {
        let entity = game_entities.allocate_get();

        // Put a position on everything
        entity.add_component(
            &mut *pos_components,
            PositionComponent::new(Vec2::new(i as f32, 6.0)),
        );

        // Put velocity on half of the entities
        if i % 2 == 0 {
            entity.add_component(
                &mut *vel_components,
                VelocityComponent::new(Vec2::new(i as f32, 6.0)),
            );
        }

        // Add an extra component to a particular entity
        if i == 4 {
            entity.add_component(
                &mut *speed_multiplier_components,
                SpeedMultiplierComponent::new(12.0),
            );
        }
    }
}

fn main() {
    // Register global systems
    let mut resource_map = ResourceMapBuilder::new()
        .with_resource(UpdateCount::new())
        .with_resource(TimeState::new())
        .with_component(<PositionComponent as Component>::Storage::new())
        .with_component(<VelocityComponent as Component>::Storage::new())
        .with_component(<SpeedMultiplierComponent as Component>::Storage::new())
        .build();

    // Create a bunch of objects
    create_objects(&resource_map);

    use minimum::resource::async_dispatch::MinimumDispatcher;
    let dispatcher = MinimumDispatcher::new(resource_map);

    dispatcher.enter_game_loop(|ctx| {
        let ctx1 = ctx.clone();

        ExecuteSequential::new(vec![
            ctx.run_task(UpdatePositions),
            //This will mutably fetch every component type so needs to be done exclusively
            ctx.visit_resources(move |resource_map| {
                let mut entity_set = resource_map.fetch_mut::<EntitySet>();
                entity_set.flush_free(resource_map);
            }),
            ctx.visit_resources_mut(move |resource_map| {
                let mut update_count = resource_map.fetch_mut::<UpdateCount>();
                println!("update {}", update_count.count);
                update_count.count += 1;
                if update_count.count > 10 {
                    ctx1.end_game_loop();
                }
            }),
        ])
    });
}
