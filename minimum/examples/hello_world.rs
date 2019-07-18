use minimum::systems::{
    DataRequirement, Read, Task, World, WorldBuilder, Write,
};

use minimum::async_dispatcher::ExecuteSequential;

use minimum::component::{Component, ComponentStorage};

use nalgebra_glm as glm;

// Keep track of how many frames we've run
struct UpdateCount {
    count: i32,
}

impl UpdateCount {
    fn new() -> Self {
        return UpdateCount { count: 0 };
    }
}

// Mock a physics system
struct PhysicsSystem;
impl PhysicsSystem {
    fn update(&mut self) {}
}

// A task to trigger updating the physics system
struct UpdatePhysicsSystem;
impl Task for UpdatePhysicsSystem {
    type RequiredResources = (Write<PhysicsSystem>);

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let mut physics_system = data;
        physics_system.update();
    }
}

#[derive(Debug)]
struct PositionComponent {
    position: glm::Vec2,
}

impl PositionComponent {
    fn new(position: glm::Vec2) -> Self {
        PositionComponent { position }
    }
}

impl Component for PositionComponent {
    type Storage = minimum::component::VecComponentStorage<PositionComponent>;
}

#[derive(Debug)]
struct VelocityComponent {
    velocity: glm::Vec2,
}

impl VelocityComponent {
    fn new(velocity: glm::Vec2) -> Self {
        VelocityComponent { velocity }
    }
}

impl Component for VelocityComponent {
    type Storage = minimum::component::SlabComponentStorage<VelocityComponent>;
}

#[derive(Debug)]
struct SpeedMultiplierComponent {
    multiplier: f32,
}

impl SpeedMultiplierComponent {
    fn new(multiplier: f32) -> Self {
        SpeedMultiplierComponent { multiplier }
    }
}

impl Component for SpeedMultiplierComponent {
    type Storage = minimum::component::SlabComponentStorage<SpeedMultiplierComponent>;
}

struct TimeState {
    dt: f32,
}

impl TimeState {
    fn new() -> Self {
        TimeState { dt: 1.0 / 60.0 }
    }
}

struct GameEntities {
    set: minimum::entity::EntitySet,
}

impl GameEntities {
    // Install component storage as needed to the world, create an entity_set, register
    // component types to that set
    pub fn setup(world: &mut World) {
        // The ctors here can take parameters as config (for example to hint max counts for each type)
        world.insert(<PositionComponent as Component>::Storage::new());
        world.insert(<VelocityComponent as Component>::Storage::new());
        world.insert(<SpeedMultiplierComponent as Component>::Storage::new());

        let mut entity_set = minimum::entity::EntitySet::new();
        entity_set.register_component_type::<PositionComponent>();
        entity_set.register_component_type::<VelocityComponent>();
        entity_set.register_component_type::<SpeedMultiplierComponent>();

        let game_entities = GameEntities { set: entity_set };

        world.insert(game_entities);
    }

    pub fn update(world: &World) {
        let mut entity_set = world.fetch_mut::<GameEntities>();
        entity_set.set.flush_free(&world);
    }
}

struct UpdatePositions;
impl Task for UpdatePositions {
    type RequiredResources = (
        Read<TimeState>,
        Read<GameEntities>,
        Read<<VelocityComponent as Component>::Storage>,
        Write<<PositionComponent as Component>::Storage>,
        Read<<SpeedMultiplierComponent as Component>::Storage>,
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
        println!("entity count {:?}", game_entities.set.iter().count());
        for entity in game_entities.set.iter() {
            println!("all entities: {:?}", entity.handle());
        }

        // EXAMPLE: non-mutable iterate over entities with velocity components
        for (entity_handle, vel_component) in velocity_components.iter(&game_entities.set) {
            println!(
                "entities with velocity: E: {:?} V: {:?}",
                entity_handle, vel_component
            );
        }

        //EXAMPLE: mutable iterate over entities with position components
        for (entity_handle, pos_component) in position_components.iter_mut(&game_entities.set) {
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
        for (entity_index, pos) in position_components.iter_mut(&game_entities.set) {
            if let (Some(vel), mul) = (
                velocity_components.get(&entity_index),
                speed_multiplier_components.get(&entity_index),
            ) {
                println!("p {:?} v {:?} m {:?}", pos, vel, mul);
                pos.position +=
                    time_state.dt * vel.velocity * mul.map(|x| x.multiplier).unwrap_or(1.0);
            }
        }
    }
}

fn create_objects(world: &World) {
    let mut game_entities = world.fetch_mut::<GameEntities>();
    let mut pos_components = world.fetch_mut::<<PositionComponent as Component>::Storage>();
    let mut vel_components = world.fetch_mut::<<VelocityComponent as Component>::Storage>();
    let mut speed_multiplier_components =
        world.fetch_mut::<<SpeedMultiplierComponent as Component>::Storage>();

    for i in 0..10 {
        let entity = game_entities.set.allocate_get();

        // Put a position on everything
        entity.add_component(
            &mut *pos_components,
            PositionComponent::new(glm::vec2(i as f32, 6.0)),
        );

        // Put velocity on half of the entities
        if i % 2 == 0 {
            entity.add_component(
                &mut *vel_components,
                VelocityComponent::new(glm::vec2(i as f32, 6.0)),
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

fn simple_implementation(world: World) {

    use minimum::systems::simple_dispatch::MinimumDispatcherBuilder;
    let dispatcher = MinimumDispatcherBuilder::from_world(world).build();

    dispatcher.enter_game_loop(|ctx| {
        ctx.run_task(UpdatePhysicsSystem);
        ctx.run_task(UpdatePositions);

        {
            let world = ctx.world();
            GameEntities::update(&world);
        }

        {
            let world = ctx.world();
            let mut update_count = world.fetch_mut::<UpdateCount>();
            println!("update {}", update_count.count);
            update_count.count += 1;
            if update_count.count > 10 {
                ctx.end_game_loop();
            }
        }

    });
}

fn async_implementation(world: World) {
    use minimum::systems::async_dispatch::MinimumDispatcherBuilder;
    let dispatcher = MinimumDispatcherBuilder::from_world(world).build();

    dispatcher.enter_game_loop(|ctx| {
        let ctx1 = ctx.clone();
        let ctx2 = ctx.clone();

        ExecuteSequential::new(vec![
            ctx.run_task(UpdatePhysicsSystem),
            ctx.run_task(UpdatePositions),
            //This will mutably fetch every component type so needs to be done exclusively
            Box::new(futures::lazy(move || {
                let world = ctx1.world();
                GameEntities::update(&world);
                Ok(())
            })),
            Box::new(futures::lazy(move || {
                let world = ctx2.world();
                let mut update_count = world.fetch_mut::<UpdateCount>();
                println!("update {}", update_count.count);
                update_count.count += 1;
                if update_count.count > 10 {
                    ctx.end_game_loop();
                }

                Ok(())
            })),
        ])
    });
}

fn main() {
    // Register global systems
    let mut world = WorldBuilder::new()
        .with_resource(PhysicsSystem)
        .with_resource(UpdateCount::new())
        .with_resource(TimeState::new())
        .build();

    GameEntities::setup(&mut world);
    create_objects(&world);

    simple_implementation(world);
    //async_implementation(world);
}
