use legion::*;
use glam::Vec2;

struct PositionComponent(pub glam::Vec2);
struct VelocityComponent(pub glam::Vec2);
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

    // Create a legion world
    let mut world = World::default();
    let mut resources = Resources::default();

    // Insert a resource that can be queried to find gravity
    resources.insert(Gravity(-9.8 * Vec2::unit_y()));

    // Insert an object with position and velocity
    let entity = *world
        .insert(
            (),
            (0..1).map(|_| {
                (
                    PositionComponent(Vec2::new(0.0, 500.0)),
                    VelocityComponent(Vec2::new(5.0, 0.0)),
                )
            }),
        )
        .first()
        .unwrap();

    for _ in 0..10 {
        // Fetch gravity... and integrate it to velocity.
        let gravity = resources.get::<Gravity>().unwrap();
        let query = <Write<VelocityComponent>>::query();
        for mut vel in query.iter_mut(&mut world) {
            vel.0 += gravity.0;
        }

        // Iterate across all entities and integrate velocity to position
        let query = <(Write<PositionComponent>, TryRead<VelocityComponent>)>::query();
        for (mut pos, vel) in query.iter_mut(&mut world) {
            if let Some(vel) = vel {
                pos.0 += vel.0;
            }

            pos.0 += gravity.0;
        }

        let position = world.get_component::<PositionComponent>(entity).unwrap();
        println!("Position is {}", position.0);
    }
}
