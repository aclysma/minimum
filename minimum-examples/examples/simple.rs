use minimum::component::{Component, ComponentStorage};
use minimum::{EntitySet, WriteAllTask, WriteAllTaskImpl, DispatchControl};
use minimum::WorldBuilder;
use minimum::Read;
use minimum::Write;
use minimum::ResourceTask;
use minimum::ResourceTaskImpl;
use minimum::DataRequirement;
use minimum::TaskConfig;
use minimum::ResourceMap;
use minimum::TaskContextFlags;

mod shared;

use shared::components::{PositionComponent, SpeedMultiplierComponent, VelocityComponent};

use shared::resources::{TimeState, UpdateCount};

use shared::Vec2;
use minimum::world::UpdateLoopSingleThreaded;

pub struct UpdatePositions;
pub type UpdatePositionsTask = ResourceTask<UpdatePositions>;
impl ResourceTaskImpl for UpdatePositions {
    type RequiredResources = (
        Read<TimeState>,
        Read<EntitySet>,
        Read<<VelocityComponent as Component>::Storage>,
        Write<<PositionComponent as Component>::Storage>,
        Read<<SpeedMultiplierComponent as Component>::Storage>,
    );

    fn configure(task_config: &mut TaskConfig) {
        task_config.this_runs_during_phase::<minimum::task::PhasePhysics>();
    }

    fn run(
        context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow
    ) {
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

pub struct UpdateEntitySet;
pub type UpdateEntitySetTask = WriteAllTask<UpdateEntitySet>;
impl WriteAllTaskImpl for UpdateEntitySet {
    fn configure(task_config: &mut TaskConfig) {
        task_config.this_runs_during_phase::<minimum::task::PhaseEndFrame>();
    }

    fn run(context_flags: &TaskContextFlags, resource_map: &mut ResourceMap) {
        let mut entity_set = resource_map.fetch_mut::<minimum::EntitySet>();
        entity_set.flush_free(&resource_map);
    }
}

pub struct IncrementUpdateCount;
pub type IncrementUpdateCountTask = ResourceTask<IncrementUpdateCount>;
impl ResourceTaskImpl for IncrementUpdateCount {
    type RequiredResources = (
        Write<UpdateCount>,
        Write<DispatchControl>
    );

    fn configure(task_config: &mut TaskConfig) {
        task_config.this_runs_during_phase::<minimum::task::PhaseEndFrame>();
    }

    fn run(context_flags: &TaskContextFlags, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (mut update_count, mut dispatch_control) = data;

        println!("update {}", update_count.count);
        update_count.count += 1;
        if update_count.count > 10 {
            dispatch_control.end_game_loop();
        }
    }
}


//TODO: Rewrite to use an entity prototype
fn create_objects(resource_map: &ResourceMap) {
    let mut game_entities = resource_map.fetch_mut::<minimum::EntitySet>();
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
    let world = WorldBuilder::new()
        .with_resource(UpdateCount::new())
        .with_resource(TimeState::new())
        .with_component(<PositionComponent as Component>::Storage::new())
        .with_component(<VelocityComponent as Component>::Storage::new())
        .with_component(<SpeedMultiplierComponent as Component>::Storage::new())
        .with_task::<UpdatePositionsTask>()
        .with_task::<UpdateEntitySetTask>()
        .with_task::<IncrementUpdateCountTask>()
        .build();

    // Create a bunch of objects
    create_objects(&world.resource_map);

    let update_loop = UpdateLoopSingleThreaded::new(world, 0);
    update_loop.run();
}
