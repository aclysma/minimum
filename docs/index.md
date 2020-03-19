
# Minimum Documentation

## Philosophy

Most game engines are a big wad of code that (hopefully) implements everything one would ever want for a game. For
example, UE4 and Unity provide their own prescribed systems (for rendering, audio, etc.) along with a way to glue custom
game code with their systems.

The primary goal of this "engine" is to avoid packing in prescribed solutions, for at least two reasons:
 * Scope Management - We don't have the bandwidth to implement or maintain everything ourselves
 * Quality - It's better to let you pick exactly what you need rather that prescribing the same solution
   for everyone 

However, finding a way for people using slightly different solutions to share code is challenging. Minimum aims
to solve this by defining a "protocol" for game data and an extendable toolset to work with it. (We prefer the word
"kernel" over "engine")

Our hope is that this approach will lead to an ecosystem of easy-to-share solutions with well-managed scope. We believe
such an ecosystem would be more compatible with the OSS model than a monolithic design. It lets people own just what
they're interested in and decentralizes technical decisions - a requirement for sustainable growth.

## How Extension Works

In order for systems to inter-operate without being strongly coupled, we are embracing data-driven design. Rather than
have one system directly call functions on another system, we expect systems to read/write data on resources and
components. It is only necessary to share the format of the data - not any of the implementation that produces or
consumes the data.

Example: Any physics system could write a matrix into the transform component, and any renderer could read the matrix from
the transform component. 
```
[Physics System] -> Transform Component -> [Rendering System]
```

This avoids dependencies between the physics and rendering systems, allowing downstream crates to be owned by different
authors.

## Tutorial

Since our goal is to empower you to choose whatever systems you want, our tutorial is a bit non-linear. Here's the
outline:

 * Development Environment Setup
   * How to init your own crate
   * How to install dependencies
 * Windowing
   * SDL2
   * winit
 * Rendering
   * SDL2
   * skia
 * Physics
   * nphysics
   * Box2d

## Crate Descriptions / Hierarchy

 * minimum-kernel
   * Starting the asset daemon (see atelier-assets)
   * Component Registration - Allows registering of component types - which in turns generates the necessary code for
     working with components (i.e. creating, diffing, serializing, etc.)
   * Prefabs (Importing/Cooking/Saving)
 * minimum-transform
   * A general system for tracking position/rotation/scaling of entities
 * minimum-math
   * Wrappers around glam math types that make them imgui-friendly      
 * minimum-game
   * Protocols for basic game systems like:
     * Input
     * Time Keeping
     * Camera/Viewport State
     * Accessing the legion universe
     * Acessing the ImGui Context/Ui
 * minimum-editor
   * Inspect Registry (i.e. property editor compatibility)
   * Selection Registry (i.e. detecting if an entity is clicked/dragged over)
   * ImGui Windows
   * "Editor Draw" - an immediate-mode world-space drawing system (for use by tools)
   * Some basic editor-draw compatible gizmos for modifying entity transform state
   * State about currently opened prefab, undo/redo queue, etc.
