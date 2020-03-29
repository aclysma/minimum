
# SDL2 Example

This is a walkthrough of how the SDL2 example works. This example demonstrates using SDL2, nphysics, and skia to do a
few common tasks that would be required to make a physics-enabled game. SDL2 and skia were chosen for their maturity
and stability. nphysics was chosen as an easy to use and build component that seems to be popular in the rust community.

I don't think the current method of integrating these together is the best, so rather than detail it at length, I'll
just list out what's there.

 * main.rs - Sets up logging, starts the daemon, runs the game
 * lib.rs - Opens a window, inits the renderer and other systems. (see `create_resources()`). Then runs an update loop
 * registration.rs - There is some registration and configuration needed for components.
     * `create_asset_manager()` - Enumerates all the assets that could be loaded
     * `create_component_registry()` - Enumerates all component types and how they are spawned
     * `create_editor_selection_registry()` - Registers the components that can be selected by clicking on them
     * `create_editor_inspector_registry()` - Registers components that can be added/removed/modified in the inspector
 * systems/mod.rs - Produces the update loop schedule. Many of the functions referred to here are in other upstream
   modules
 * systems/app_control_systems.rs - A system that exits when escape is hit
 * systems/draw_systems.rs - A system that sends data that needs to be drawn to skia

The main non-ideal issue here is that you have to register all the components, assets, and systems manually that are 
contained in upstream code. There would ideally be a way to refer to a single idiomatic thing in upstream code.

However, this does demonstrate the potential of a thin kernel hosting major features that can be mixed and matched by
and end-user.