# minimum

A game development framework that provides basic tooling and a content authoring workflow. Minimum has easy-to-use 
extension points for integrating custom and 3rd-party libraries with your game logic.

This library is best suited for use by those who want to start with something thin and bring their own tech to put on 
top of it. It's your very own build-a-game-engine toolkit.

## Status

This project remains a proof-of-concept, although it has gone through a major rework to rely on legion for ECS and
atelier for asset loading/processing. The goal remains to produce a lean "kernel" that allows many different upstream
libraries to inter-operate.

## Features

Editing functionality is currently limited, but the core loop is implemented:
 * Entities with components can be loaded from file and saved to file
 * Entities can be selected and their components can be edited (at design-time and run-time)
 * Entities and components can be added or removed
 * Entities can be moved, scaled, and rotated
 * Can start/stop/reset the simulation
 
## Philosophy

Game engines are usually huge - and this poses difficulty for OSS projects. It's very difficult for a large OSS project
to have a unified focus and vision. This is especially true in games where potential contributors may want very
different things out of the engine that they are trying to collaborate on.

By reducing the eponymous "engine" to a kernel, we can push these decisions further down the chain. This allows us to
share both the kernel as well as the upstream integrations - since those are opt-in. There are other benefits too - it
eases distributed development by allowing many people to own small pieces. This flexibility also means that contributors
can choose the work that fits their interest and skill set.

To achieve interoperability, we will need a common protocol for these integrations to work well together. So we will
standardize on a common base set of components, resources, and systems. For example, a common transform component, or
a common way to represent input state (it would be up to the downstream user to pick an input/windowing implemnetation
that populates this.)

## Alternatives

For more batteries-included solutions in rust, I would look at amethyst, coffee, or ggez. The main difference is that
these libraries all tend to take over your game loop or assume you will use a particular windowing or rendering
solution.

minimum requires that you bring your own renderer (including support for imgui.) However, this also gives you the
flexibility to choose your own solutions.

## Directory Map

/contrib - Holds integrations for upstream libraries like nphysics, sdl2, and winit
/docs - Some concept-level documentation
/examples
 * example-sdl2 - A working implementation using SDL2 for windowing
 * example-winit - A working implementation using winit for windowing
 * example-shared - Shared code between the examples that isn't useful outside the examples
 * tutorial/examples - Completed code samples from the tutorial in docs
/minimum-editor - Editing logic that you do not need to ship in your game
/minimum-game - Common protocol for game-level logic that would ship
/minimum-kernel - Legion, atelier, and prefab integration
/minimum-math - Some wrappers around glam math types to make them friendly with inspection
/minimum-transform - Protocol/structure that defines placement of objects

## Running the Demo

```
git clone https://github.com/aclysma/minimum.git
cd minimum
cd demo
cargo run --example example-sdl2
```

## Roadmap

TBD.. I plan to dogfood this for personal projects to help find gaps in needed functionality.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

The fonts directory contains several fonts under their own licenses:
 * [Feather](https://github.com/AT-UI/feather-font), MIT
 * [Material Design Icons](https://materialdesignicons.com), SIL OFL 1.1
 * [FontAwesome 4.7.0](https://fontawesome.com/v4.7.0/license/), available under SIL OFL 1.1
 * [`mplus-1p-regular.ttf`](http://mplus-fonts.osdn.jp), available under its own license.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).
