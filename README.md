# minimum

A game development framework that provides basic tooling and a content authoring workflow. Minimum has easy-to-use 
extension points for integrating custom and 3rd-party libraries with your game logic.

This library is best suited for use by those who want to start with something thin and bring their own tech to put on
top of it. It's your very own build-a-game-engine toolkit.

[![Build Status](https://travis-ci.org/aclysma/minimum.svg?branch=master)](https://travis-ci.org/aclysma/minimum)

## Status

This framework is a proof-of-concept. Some major changes I'd like to make:
 * Replace much of the base/ECS layer with `legion`. (Legion was missing Read/Write awareness for multi-threading purposes.)
 * Use `atelier` for asset loading/processing.
 
Some work towards both changes can be found [here](https://github.com/aclysma/atelier-legion-demo). I'm hoping to bring
these changes into minimum, but it could be a while.

## Features

Editing functionality is currently limited, but the core loop is implemented:
 * Entities with components can be loaded from file and saved to file
 * Entities can be selected and their components can be edited (at design-time and run-time)
 * Entities and components can be added or removed
 * Entities can be moved, scaled, and rotated
 * Can start/stop/reset the simulation
 
Youtube Video:

[![IMAGE ALT TEXT](http://img.youtube.com/vi/BON_RvVFiWY/0.jpg)](https://www.youtube.com/watch?v=BON_RvVFiWY "Video of Editor in Use")

## Platform Support
 * The base/ECS is very portable. It's pure rust and has few upstream dependencies. `no_std` (for this 
   portion of the library only) builds but is not yet tested. To use `no_std`, disable default features and 
   don't use feature `std` 
 * The demo builds for Windows/MacOS/Linux. Only being tested on MacOS for now. Pull requests to
   improve support for other platforms very welcome!

The no-editor build of this library could realistically support embedded (`no_std`), mobile, PC, and web platforms 
because this avoids coupling to a renderer/windowing system. For the time being, the focus is on PC.

## Alternatives
 * For a mature ecs, I suggest looking at `shred` or `specs`. (In fact some of this library is quite similar to shred!)
   `legion` is also a great choice and is improving rapidly 
 * For more batteries-included solutions in rust, I would look at `amethyst`, `coffee`, or `ggez`.
     * I expect a typical use-case one day would be to combine this framework with another "engine" that is focused more
       on functionality than tooling/workflow.

## Directory map

 * [/minimum](minimum-base) - A lightweight ECS and update loop system
 * [/minimum-framework](minimum-framework) - More opinionated framework built on top of the ECS to provide a good
   tooling and workflow baseline
 * [/minimum-demo](minimum-demo) - An example project that demonstrates integrating minimum with several popular
   libraries from within the rust ecosystem  
 * [/minimum-examples](minimum-examples) - A small collection of sample code to demonstrate usage

## Running the Demo

[/minimum-demo](minimum-demo) shows a more realistic integration of these utilities with other popular 
libraries like `winit`, `gfx-hal`, `rendy`, `nphysics`, and `imgui`. It would be a reasonable template for something
small, and it shows how the pieces provided could be fit together for something bigger.

Over time, the functionality that isn't coupled to these libraries will move to [/minimum-framework](minimum-framework)

To run the demo:
 * Working directory must be `/minimum-demo`
 * The `editor` feature (will likely rename to tools later...) enables an editor. It is ON by default since this is a demo!
 * Use feature `dim2` or `dim3` depending on if you want to use 2d or 3d physics
 * Use `metal`, `dx12`, or `vulkan` feature when using cargo commands
     * Example: `cargo run --features="metal editor dim2"`

## Roadmap

In no particular order:
 * Another pass on the tasking/dispatching logic
 * Core editing functionality: copy/paste, undo/redo, parenting, etc.
 * Extract editing functionality out of the demo to a new sub-crate
 * Continue building the demo to do more interesting physics, rendering, and gameplay logic
 * API-level and conceptual-level documentation

## Contribution

All contributions are assumed to be dual-licensed under MIT/Apache-2.

## License

Distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).

The demo project uses several fonts under their own licenses
 * [Feather](https://github.com/AT-UI/feather-font), MIT
 * [Material Design Icons](https://materialdesignicons.com), SIL OFL 1.1
 * [FontAwesome 4.7.0](https://fontawesome.com/v4.7.0/license/), available under SIL OFL 1.1
 * [`mplus-1p-regular.ttf`](http://mplus-fonts.osdn.jp), available under its own license.
