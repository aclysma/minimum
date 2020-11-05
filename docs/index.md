
# Minimum Documentation

## Tutorials/Examples

 * Prefab System ([Complete code for each tutorial here!](../examples/tutorial))
   * [Dev Environment Setup](tutorial/tutorial001_dev_environment_setup.md)
   * [Creating Prefabs](tutorial/tutorial002_creating_prefabs.md)
   * [Saving and Loading Prefabs](tutorial/tutorial003_save_and_load_prefabs.md)
   * [Cooking Prefabs](tutorial/tutorial004_cooking_prefabs.md)
   * [Spawning Prefabs](tutorial/tutorial005_spawning_prefabs.md)
   * [Transactions](tutorial/tutorial006_transactions.md)
   * [Creating Prefab Overrides](tutorial/tutorial007_creating_prefab_overrides.md)
 * TODO: More tutorials.. in the meantime please see the examples!
   * [Winit](..examples/example-winit)
   * [SDL2](../examples/example-sdl2)

### Running the Examples

```
# Fetch the repo
git clone https://github.com/aclysma/minimum.git
cd minimum

# These are not technically examples so use run --package
# Make sure the current directory is "examples" so assets can be found/loaded! 
cd examples
cargo run --package example-winit
cargo run --package example-sdl2

# These *are* examples - one per above tutorial
cd tutorial
cargo run --example tutorial001_dev_environment_setup
```

## Philosophy

Most game engines are vertical integrations of every system one would want for a game. For example, UE4 and Unity
provide their own prescribed systems (for rendering, audio, etc.) along with a way to glue custom game code with their
systems.

The primary goal of this (experimental) "engine" is to avoid packing in prescribed solutions, for at least two reasons:
 * Scope Management - We don't have the bandwidth to implement or maintain everything ourselves
 * Quality - It's better to let you pick exactly what you need rather that prescribing the same solution
   for everyone 

However, finding a way for people using slightly different solutions to share code is challenging. Minimum aims
to solve this by defining a "protocol" for game data and an extendable toolset to work with it. (We prefer the word
"kernel" over "engine")

The hope is that this approach can support to an ecosystem of easy-to-share solutions with well-managed scope. I think
such an ecosystem would be more compatible with the OSS model than a monolithic design. It lets people own just what
they're interested in and decentralizes technical decisions - a requirement for sustainable growth.

## How Extension Works

Rather than have one system directly call functions on another system, systems read/write data on resources and
components. It is only necessary to share the format of the data - not any of the implementation that produces or
consumes the data.

Example: Any physics system could write a matrix into the transform component, and any renderer could read the matrix from
the transform component. 
```
[Physics System] -> Transform Component -> [Rendering System]
```

This avoids dependencies between the physics and rendering systems, allowing downstream crates to be owned by different
authors. The data is essentially the interface between the systems.
