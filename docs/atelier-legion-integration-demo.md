# Atelier/Legion Integration Demo

This post originally appeared on https://community.amethyst.rs/t/atelier-legion-integration-demo/1352

Much of the code in the demo was ported from minimum 0.1, and the techniques explored in the demo have since been rolled back into minimum.

---

In the last few months, @kabergstrom and I have made some good progress on exploring how [legion](https://github.com/TomGillen/legion) and [atelier](https://github.com/amethyst/atelier-assets) could be integrated together, particularly in the context of implementing an editing workflow. Some of the goals we started with are here: [Rich Editing Experience Using atelier-assets and legion](https://community.amethyst.rs/t/rich-editing-experience-using-atelier-assets-and-legion/1263)

We now have a working prototype that implements interactive editing, including support for undo and redo.

Video: https://www.youtube.com/watch?v=9Vwi29RuQBE&feature=youtu.be

Demo Github: https://github.com/aclysma/atelier-legion-demo.git

Most of the functionality has been factored out into a few reusable crates here: https://github.com/kabergstrom/prefab

We believe that the general architecture in the demo could be a good fit for amethyst. Further, the general approach could probably be shared with the broader Rust gamedev community.

This document demonstrates what using the API feels like and discusses the lessons learned during implementation.

As a quick feature run-down, the demo supports:

* Loading and saving the edited state as a prefab
* Hot-reloading the edited state when the file changes
* Undo and Redo
* Creating/Deleting entities
* Adding/Removing components
* Multiple selection
* Drag to transform, scale, and rotate
* Rendering and physics components respect transform, scale, and rotate component data.

In order to try the demo for yourself, you can do something like this:

    git clone https://github.com/aclysma/atelier-legion-demo.git
    cd atelier-legion-demo
    git submodule init
    git submodule update
    cargo run

# Cloneable and Non-Cloneable Components

The demo contains components that represent position, rotation, and scaling. Additionally, it has a few components for rendering and physics.

These components can be divided into two categories - cloneable and non-cloneable.

![|Component Transform Example](component_transform_example.jpeg)

The components in this image are examples and don’t necessarily match the demo

## Cloneable Components

Cloneable components must implement a number of traits:

* Clone - Used for cloning edit-time data into the runtime world
* Serialize/Deserialize - Used for saving/loading data in the editor
* Default - Used when adding a component to an entity to determine initial state
* PartialEq - Required for SerdeDiff...
* SerdeDiff - SerdeDiff lets us determine what changed in user-edited data
* TypeUuid - In our implementation, we use UUIDs to identify types in a registry.

If a structure implements the above traits, then it can be used to represent a component at all stages of the pipeline - editing, loading, and running.

One example of this would be a Position component. It might be defined like this:

    #[derive(
        Clone, Serialize, Deserialize, Default, 
        PartialEq, SerdeDiff, TypeUuid
    )]
    #[uuid = "8bf67228-f96c-4649-b306-ecd107194cf0"]
    Struct PositionComponent {
        position: Vec3
    }

The PositionComponent structure defines the format of positions at edit/load time as well as at runtime in the shipped game.

## Non-Cloneable Components

Sometimes, components at runtime won’t be able to implement some of these traits. A common case is data within the component that is not cloneable or serializable. For example, a physics component that represents a rigid body attached to the entity might need a handle to the rigid body in a physics engine.

This clearly can’t be serialized as it would be meaningless once deserialized. Further, the data required to describe how to create the rigid body is not the same as the data required to represent the rigid body once it has been created.

In order to support this, we have the concept of ComponentDefinitions. The ComponentDefinition is used for editing and loading and the actual Component is only used in the game world at runtime.

    // This is the edit-time representation, it must support
    // a number of traits
    #[derive(
        Clone, Serialize, Deserialize, Default, 
        PartialEq, SerdeDiff, TypeUuid
    )]
    #[uuid = "8bf67228-f96c-4649-b306-ecd107194cf1"]
    pub struct RigidBodyBallComponentDef {
       pub radius: f32,
       pub is_static: bool,
    }
    
    // Runtime data, no derives required for this!
    pub struct RigidBodyComponent {
       pub handle: DefaultBodyHandle,
       delete_body_tx: crossbeam_channel::Sender<DefaultBodyHandle>,
    }

In this example, the handle points to a rigid body stored in an nphysics world.

It’s important to note here, for non-cloneable components, the end-user does not edit actual component data - they edit the component definition. So from an end-user’s perspective, they just see `radius` and `is_static` as editable fields.

### Transforming from Definitions to Components

We support three methods for a downstream user to transform a component definition into a real components:

#### Via Into/From

If an end-user implements the standard library’s Into or From, the components can simply be registered like this:

    registry.add_mapping_into::<RigidBodyBallComponentDef, RigidBodyComponent>();

The engine will do the equivalent of... 

    let component = component_definition.into();

...to spawn the component

#### Via SpawnInto/SpawnFrom

While using Into/From is convenient and familiar to many, it does not allow us to pass extra data for the end user’s implementation to use. In order to support this, we have parallel traits SpawnInto/SpawnFrom that mimic Into/From, but allow the end user to access other entity or system data.

    /// Trait for mapping one type into another with world.clone_from()
    pub trait SpawnInto<IntoT: Sized>
    where
       Self: Sized,
    {
       fn spawn_into(
           src_world: &World,
           src_component_storage: &ComponentStorage,
           src_component_storage_indexes: Range<ComponentIndex>,
           resources: &Resources,
           src_entities: &[Entity],
           dst_entities: &[Entity],
           from: &[Self],
           into: &mut [MaybeUninit<IntoT>],
       );
    }

One expected usage would be looking up position/rotation/scaling components on the entity to properly place the rigid body into the world. Another expected usage includes looking up a system. In the case of a rigid body component, we might need write access to the physics world so that we can insert the rigid body and get a handle to it.

The SpawnInto/SpawnFrom traits operate on blocks of data at a time. For example, if a prefab contains many entities with RigidBody components, construction of those components can be batched.

### Via Closure

We also support using a closure, equivalent to the SpawnFrom/SpawnInto trait

    clone_merge_impl.add_mapping_closure::<RigidBodyBoxComponentDef, RigidBodyComponent>(|
       src_world,
       src_component_storage,
       src_component_storage_indexes,
       resources,
       src_entities,
       dst_entities,
       src_data,
       dst_data
    | {
       // implement here
    };

In practice, since the closure needs to take so many parameters, using the SpawnFrom/SpawnInto trait ends up being cleaner. However, it could be useful for cases where it’s not possible to implement the trait due to Rust’s orphaning rules.

# Diffing Components

In order to implement the demo, we needed a way to detect and reproduce changes to data in a general way. We published a new crate, [serde-diff](https://github.com/amethyst/serde-diff.git), that allows two instances of the same struct to be compared, resulting in a Diff. The Diff is serializable/deserializable and can be re-applied to other instances of the same struct. This is used for two major features - transactions and prefabs.

## Transactions

In the demo, the editing tools do not modify world data directly. Instead, when a tool needs to modify state, it creates a transaction. Any entities that will be modified are added to the transaction. This allows the transaction to copy entity data into two new legion worlds - a baseline “before” state that is immutable, and the mutable “after” state.

    // Example of creating a transaction
    let tx = TransactionBuilder::new()
        .add_entity(entity_one)
        .add_entity(entity_two)
        .begin(world, clone_impl);

Most of the time the transaction will just operate on whatever is selected in the editor. In the demo, we have a shorthand for this.

    // Create a transaction - this helper function adds all 
    // the selected entities to the transaction
    let tx = editor_ui_state.create_transaction_from_selected(
        &*editor_selection,
    );

The end-user’s code can then modify the transaction’s world freely. This code that performs the edit doesn’t need to be aware of the transaction - just the transaction’s “after” legion world (likely passed via mutable ref).

    // Adding, deleting, and modifying entity data is supported here
    tx.world_mut().delete_all();

Once the end-user’s code runs, the transaction can be committed. The data owned by the transaction in the “before” and “after” worlds is compared to produce diffs. The diff data supports creating/destroying entities, adding/removing components, and modifying component data.

    // Now generate the necessary data to apply/revert the change
    let tx_diffs = tx.create_transaction_diffs(component_registry);

The TransactionDiffs returned by create_transaction_diffs contains an “apply” set of diffs and a “revert” set of diffs.

These diffs can be used for several purposes:

* Applying changes to the runtime state
* Sending the changes back to the atelier daemon to commit changes to disk, and possibly other connected devices
* Implementing Undo/Redo

## Prefabs

Because serde-diff allows us to capture and persist changes with per-field granularity, we can use it to implement prefabs. Prefabs can contain entities and references to other prefabs. The “prefab references” can also include diffs to be applied to components of specific entities in the referenced prefab.

Prefabs can be stored in two forms, cooked and uncooked.

### Uncooked Prefabs

Uncooked prefabs are hierarchical and editor-friendly. In memory, an uncooked prefab contains a legion world with entity data and references to other prefabs. These prefab references also contain any diffs that would need to be applied to the referenced prefab.

### Cooked Prefabs

At runtime, we don’t want to incur the cost of loading and tracking a prefab hierarchy or applying the diffs. To avoid the cost, we can flatten an uncooked prefab and all the other uncooked prefabs that it references into a single cooked prefab where all the data is stored in a legion world. Spawning such a prefab is straightforward - the data is simply cloned or transformed from the cooked prefab’s legion world into the game world.

Currently, the demo is loading uncooked prefabs and cooking them “in-game”. In the future, the cooking process would be pulled into atelier assets and the game would only interact with cooked prefabs.

When we make this change, we can still use transaction diffs. The editor can immediately apply the diffs to in-memory cooked data, allowing for a responsive editor experience. (For example, selecting entities and dragging them to change their position.) Additionally, the editor can forward the diffs to the daemon, which can apply the diffs to the uncooked form (potentially to be saved to disk.)

# Component Registration

In order for Rust to emit the necessary code for cloning, creating diffs, applying diffs, etc., there must be a registry for all component types. Registering the component will cause rustc to emit all the necessary code to support contrustructing, transforming, and diffing components.

Additionally, this component registry can be used to implement the CloneImpl trait, required to use clone_from in legion.

# Key Lessons/Techniques Learned

### Putting Everything into Legion Worlds Works Well

We are using legion for storing data in uncooked prefabs, cooked prefabs, transactions, and runtime data. In order to copy and transform data between these worlds, we developed a new “clone_from” feature in legion.

This allows us to use a relatively small amount of code to accomplish quite a lot - and any fixes/improvements/tooling can potentially benefit all stages of the pipeline. Further, the onboarding process for using the engine is quicker since once someone has working knowledge of legion, they can comfortably interact with data across the entire pipeline.

### Using Diffing/Transactions is both Powerful and Intuitive

If someone wants to implement a new tool in the demo, they only need to know how to create a transaction and apply the change to a legion world. The lowers the barrier to contributing new features and the simplicity makes it likely that the tool will be robust from the start.

Using this API provides automatic support for undo/redo, a notoriously difficult and error prone feature to implement by hand.

### Separating Creation and Runtime Data is Beneficial

It’s common that the data required to create a component is not the same as the data required to represent the component at runtime. Further, it’s common for runtime data to not be cloneable, serializable, or both. (For example, any FFI type.)

Integrating a physics engine with an ECS is a common question that comes up in a community - and often the solution is awkward. But by separating the construction data and runtime data, and providing a way to see init data for other components (like position,) we produced a clean integration between nphysics and legion.

We believe that many systems with runtime handles can be made cleaner by maintaining this separation of initialization and runtime data.

### Immediate-Mode UI is Quick and Easy to Use

For this project, we used imgui. We appreciated that we only needed to work in a single language to prototype UI, and that it’s API is straightforward to use.

In addition to imgui, we have a simple immediate-mode shape drawing API that detects clicking and dragging. This made implementing interactive widgets in world space more straightforward.

Conceivably other interactive tools could be built on top of this fairly easily, and the code that implements it won’t need to worry about coordinate systems, windowing, or input devices. It just knows that the line it asked to be drawn (using world-space coordinates) was dragged.

Generally, we believe that for an open source editor to work, it must be easy and straightforward to create tools that are efficient and reliable. New contributors need to be productive with little ramp-up time or help, and the code they produce needs to be easily readable by others. Coordinate space changes can be very tricky and removing the need to deal with this in every editor tool reduces the maintenance workload for everyone.

## Unsolved Problems/Future Work

### Editing Prefabs by Hand

While the demo technically supports nested prefabs, the format they are stored with is not very human readable. Currently, the data for specifying component overrides for an entity in a referenced prefab is stored like this:

    component_overrides: [
       (
           component_type: "f5780013-bae4-49f0-ac0e-a108ff52fec0",
           diff: r##"[ Enter(Field("position")), Value(Vec2(300.0, 300.0)) ]"##,
       ),
    ],

In this case, the component_type guid indicates that it’s a Position component, and the diff field describes a sequence of enter/value/exit commands.

While a sequence of commands is runtime performance friendly (aside from using strings), any non-trivial example becomes painful to read and write in a text editor. When saving this data in user-readable formats like RON, we would like to explore saving it in a more familiar hierarchical form. For example, `[Enter(Field(“Circle”)), Enter(Field(“Radius”)), Value(5.0)]` could be represented as `{ Circle: { Radius: 5.0 } } or { “Circle.Radius”: 5.0 }`

It should be possible to convert between commands and either of the other forms. So we believe this to be a very solvable problem, just a matter of putting in some time to implement it. To generalize, we want serde_diff to support both a human-readable format and a machine-readable format.

### Editing Prefabs with a UI

Currently, we do not support any form of prefab editing. All changes are applied directly to the opened prefab entities.

Designing a UI to work well with this is a lot of work and ended up being out of scope for this demo. Much of the groundwork to create such a UI is laid with the APIs for prefab cooking being implemented in the demo and this is an area that is ripe for experimentation and prototyping.

### Entity References

It’s not uncommon for one entity to need to reference another entity. For example, a heat-seeking missile meant to follow a player in a game might be implemented as a missile entity and a player entity, and the missile entity might have a component that needs to reference the player entity.

Currently, when we clone merge components from one legion world to another, any Entity stored in a component is naively copied by value. It still refers to an entity in the old world. Ideally, we could detect that an entity reference exists in component data and fix the copy in the new world to point to the copied entity in the same (new) world.

We don’t want to add complexity to legion’s Entity type, so we are considering adding a new type, EntityHandle. We could use the same technique as we use for asset handles where we store context data in TLS. This would allow serialization/deserialization to look up what a particular Entity reference should be changed to, and we can use a custom Clone impl to patch the reference as it is cloned.

### Sharing Editor Code

Ideally we’d like to wrap up the editing tools in this demo in a way that is reusable. (For example, dragging an entity to change the position, or just detecting that an entity has been clicked.)

As it is now, if someone wants to use the editing tools in the demo, they would need to copy the code and modify it to use their math and input libraries of choice. This will make it difficult for us as a community to share tooling improvements.

Unfortunately, the editor code needs to be aware of coordinate systems, input, and windowing code. Much of the choices for these things are opinionated and game-specific.

We haven’t done much work on this yet, but we are hoping that we can insert a layer of abstraction using a trait object to handle this. The reusable editing tools could then delegate the opinionated and game-specific logic to a trait object provided by the user.

### Saving Edited Data to Disk

Atelier currently does not support writing modified data back to disk. While we do have a plan for implementing this (see the “Prefabs” section) it requires further work in atelier to support it.