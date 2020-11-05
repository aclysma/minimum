# Spawning Prefabs

*Compiling code for this demo is located in [/examples/tutorial/examples](../../examples/tutorial/examples)*

Now that we have a cooked prefab, we can spawn it into a runtime world. Before we do that, lets return to component
registration briefly.

## Component Types

Component types can be divided into two categories:

### Cloneable Components

Cloneable components must implement a number of traits:

 * Clone - Used for cloning edit-time data into the runtime world
 * Serialize/Deserialize - Used for saving/loading data in the editor
 * Default - Used when adding a component to an entity to determine initial state
 * PartialEq - Required for SerdeDiff…
 * SerdeDiff - SerdeDiff lets us determine what changed in user-edited data
 * TypeUuid - In our implementation, we use UUIDs to identify types in a registry.

If a structure implements the above traits, then it can be used to represent a component at all stages of the pipeline - 
editing, loading, and running. Our position component is a good example of this.

### Non-Cloneable Components

Sometimes, components at runtime won’t be able to implement some of these traits. A common case is data within the 
component that is not cloneable or serializable. For example, a physics component that represents a rigid body attached 
to the entity might need a handle to the rigid body in a physics engine.

This clearly can’t be serialized as it would be meaningless once deserialized. Further, the data required to describe 
how to create the rigid body is not the same as the data required to represent the rigid body once it has been created.

In order to support this, we have the concept of ComponentDefinitions. The ComponentDefinition is used for editing and 
loading and the actual Component is only used in the game world at runtime.

```rust
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
```

In this example, the handle points to a rigid body stored in an nphysics world.

It’s important to note here, for non-cloneable components, the end-user would not edit actual component data - they edit 
the component definition. So from an end-user’s perspective, they just see radius and is_static as editable fields.

#### Registration

A typical component registration might look like this:

```rust    
minimum::ComponentRegistryBuilder::new()
   .auto_register_components()
   .add_spawn_mapping_into::<DrawSkiaCircleComponentDef, DrawSkiaCircleComponent>()
   .add_spawn_mapping_into::<DrawSkiaBoxComponentDef, DrawSkiaBoxComponent>()
   .add_spawn_mapping::<RigidBodyBallComponentDef, RigidBodyComponent>()
   .add_spawn_mapping::<RigidBodyBoxComponentDef, RigidBodyComponent>()
   .build()
```

#### Using Into/From

For the DrawSkiaCircleComponentDef/DrawSkiaBoxComponentDef (see the demo!) we can just use the rust built-in Into/From
traits.

```rust
impl From<DrawSkiaBoxComponentDef> for DrawSkiaBoxComponent {
    fn from(from: DrawSkiaBoxComponentDef) -> Self {
        DrawSkiaBoxComponent {
            half_extents: from.half_extents,
            paint: from.paint.into(),
        }
    }
}
```

In this case, a transform was required because paint is an FFI type in the skia library. It is not possible to serialize
it, so we implemented a serializable type for paint that could be used to construct the FFI type at runtime.

#### Using SpawnInto/SpawnFrom

For RigidBodyBallComponentDef and RigidBodyBoxComponentDef, we needed to access other components on the entity and
resources. This is a more complex usage, please see the demo source code for details. But in short, it's a matter of
implementing SpawnFrom or SpawnInto

impl SpawnFrom<RigidBodyBallComponentDef> for RigidBodyComponent {
    fn spawn_from(
        src_world: &World,
        src_component_storage: &ComponentStorage,
        src_component_storage_indexes: Range<ComponentIndex>,
        resources: &Resources,
        src_entities: &[Entity],
        dst_entities: &[Entity],
        from: &[RigidBodyBallComponentDef],
        into: &mut [std::mem::MaybeUninit<Self>],
    ) {
        // Implement your transform here. You'll likely read resources and components from the old world. When the
        // function returns, you must have initialized all the data in the `into` slice 
    }
}

## Spawning Components

```rust
let mut world = World::default();
let mut resources = Resources::default();

//
// Spawn the prefab in a new world.
//
let mut clone_impl_result = HashMap::default();
let mut spawn_impl = component_registry.spawn_clone_impl(&resources, &mut clone_impl_result);
world.clone_from(
    &cooked_prefab.world,
    &legion::query::any(),
    &mut spawn_impl,
);
```

The world will now contain copies of cloneable components, and the results of transforming non-cloneable components.

To find the entities in the world that correspond with the prefab UUID, we must
 * Look at the cooked prefab to determine which entity is associated with the UUID
 * Then use clone_impl_result in clone_from to find the copy that was cloned into the world
 
```rust
let cooked_entity = cooked_prefab.entities[&entity_uuid];
let world_entity = clone_impl_result[&cooked_entity];

let position = world
    .entry_ref(world_entity)
    .unwrap()
    .into_component::<PositionComponent>()
    .unwrap();
println!(
    "Position of {} is {}",
    uuid::Uuid::from_bytes(entity_uuid),
    position.value
);
```