# minimum

A collection of utilites for games - a relatively thin framework that helps knit 3rd party libraries with your game logic.

This library is aimed at people who want to start with something thin and bring their own tech to put on top of it.

 * For a mature ecs, I suggest looking at `shred` or `specs`. (In fact some of this library is quite similar to shred!)
 * For a more complete, turnkey solution in rust, I would look at `amethyst`, `coffee`, or `ggez`.

## Examples

A collection of small samples is located in [/minimum-examples](minimum-examples).

## Demo

Additionally, [/minimum-demo](minimum-demo) shows a more realistic integration of these utilities with other popular 
libraries like `winit`, `gfx-hal`, `rendy`, `nphysics`, and `imgui`. It would be a reasonable template for something
small, and it shows how the pieces provided could be fit together for something bigger.

**NOTE: Demo is currently using winit 0.20, so some patches are required to build it.**

```
rendy = { git = 'https://github.com/aclysma/rendy.git', branch="winit-0.20.0" }
gfx-hal = { git = 'https://github.com/aclysma/gfx.git', branch="winit-0.20.0" }
gfx-backend-metal = { git = 'https://github.com/aclysma/gfx.git', branch="winit-0.20.0" }
imgui = { git = 'https://github.com/aclysma/imgui-rs.git' }
imgui-winit-support = { git = 'https://github.com/aclysma/imgui-rs.git' }
```

## Contribution

All contributions are assumed to be dual-licensed under MIT/Apache-2.

## License

Distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).

The demo project uses [`mplus-1p-regular.ttf`](http://mplus-fonts.osdn.jp), available under its own license.
