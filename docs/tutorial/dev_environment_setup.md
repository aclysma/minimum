
# Dev Environment Setup

In order to get things started, you'll need to create a new crate:

```
cargo new --bin your_crate_name
```

Next, we need to add some dependencies to the .toml

```toml
# The minimum game engine kernel
minimum = { path = "../.." }

# Asset Pipeline
atelier-assets = { git = "https://github.com/amethyst/atelier-assets" }

# Prefab/Transactions
legion-transaction = { git = "https://github.com/kabergstrom/prefab" }
legion-prefab = { git = "https://github.com/kabergstrom/prefab" }
prefab-format = { git = "https://github.com/kabergstrom/prefab" }

# Legion ECS
legion = { git = "https://github.com/kabergstrom/legion", default-features = false, features = ["serialize"], branch="atelier-legion-demo" }

# Required for serializing/desieralizing components
serde = "1"
uuid = "0.8"
type-uuid = "0.1"
itertools = "0.8"

# Identifies diffs between two structs, used when creating transactions
serde-diff = "0.3"

# Logging
log="0.4"
env_logger = "0.6"

# Not required, but we'll use it for math in the tutorial
glam = "0.8.6"
```

Now, we need to set up a couple things

```rust
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
}
```

This enables logging and starts the asset daemon. If you wanted to customize where on disk assets are being pulled from,
you could look at the implementation of `minimum::daemon::run()`. This implementation uses command line args, but you
could implement this yourself if you prefer different defaults.

At this point you can start working with legion and many of the features we've built on top of it.

Normally, the next thing you'd do is create a window (with SDL2 or winit, for example) but for now lets just take a
short tour of what we can do with legion.

For a practical example, lets create some entities with position and velocity, and a resource that holds gravity.

First lets add some types for the math

```rust
use glam::Vec2;

struct Position(pub glam::Vec2);
struct Velocity(pub glam::Vec2);
struct Acceleration(pub glam::Vec2);
struct Gravity(pub glam::Vec2);
```


Next, we need to start up legion and register components.

```rust
// This import grabs most of the things you'd want in legion
use legion::prelude::*;

// Create a legion universe world, and resources
let universe = Universe::new();
let mut world = universe.create_world();
let mut resources = Resources::default();
```

What is this stuff?
 * Universe - You generally want exactly one of these in any application using legion.. you use it to create worlds
 * World - This is a set of entities and the components that are attached to them
 * Resources - A resource is a bit like a hash map of globals. Elements can be read and written by type. For example:

```rust
let gravity = resources.get::<Gravity>();
let mut gravity = resources.get_mut::<Gravity>();
```

These calls return a Fetch<T> or FetchMut<T> of your type. These are a bit like holding a lock on the value. It is
essentially overriding the borrow checker and moving the checking it would be doing to runtime.

The standard borrowing rules apply, if they are violated you'll get a panic. There's also a bit of overhead induced
to do the lookup. Legion has a solution for both issues, but we'll hold off on that for now.

Now we can insert a gravity resource and create some entities:

```rust
// Insert a resource that can be queried to find gravity
resources.insert(Gravity(-9.8 * Vec2::unit_y()));

// Insert an object with position and velocity
let entity = *world.insert(
    (),
    (0..1).map(|_| (Position(Vec2::new(0.0, 500.0)), Velocity(Vec2::new(5.0, 0.0))))
).first().unwrap();
```

The code for this is a bit busy, but realistically we'd be wanting to spawn this from prefabs defined by a file and
created by an editor.

And now lets write some code to integrate acceleration from gravity into velocity, and velocity into position

```rust
for _ in 0..10 {
    // Fetch gravity... and integrate it to velocity.
    let gravity = resources.get::<Gravity>().unwrap();
    let query = <(Write<Velocity>)>::query();
    for mut vel in query.iter_mut(&mut world) {
        vel.0 += gravity.0;
    }

    // Iterate across all entities and integrate velocity to position
    let query = <(Write<Position>, TryRead<Velocity>)>::query();
    for (mut pos, vel) in query.iter_mut(&mut world) {
        if let Some(vel) = vel {
            pos.0 += vel.0;
        }

        pos.0 += gravity.0;
    }

    let position = world.get_component::<Position>(entity).unwrap();
    println!("Position is {}", position.0);
}
```

You should see something like this print out:

```
Position is [5, 480.40002]
Position is [10, 451.00003]
Position is [15, 411.80005]
Position is [20, 362.80005]
Position is [25, 304.00006]
Position is [30, 235.40005]
Position is [35, 157.00005]
Position is [40, 68.80004]
Position is [45, -29.199963]
Position is [50, -136.99997]
```
