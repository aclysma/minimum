
# Creating Prefabs

In this chapter, we'll convert a legion world into a prefab.

## Setting Up Components

In the prior example, we use a couple raw structs for components. In order to use the features we will be covering in
this tutorial, these must be changed:

```
use type_uuid::TypeUuid;
use serde::Serialize;
use serde::Deserialize;
use serde_diff::SerdeDiff;

#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107190000"]
pub struct PositionComponent {
    #[serde_diff(opaque)]
    pub value: Vec2,
}

legion_prefab::register_component_type!(PositionComponent);

#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107190001"]
pub struct VelocityComponent {
    #[serde_diff(opaque)]
    pub value: Vec2,
}

legion_prefab::register_component_type!(VelocityComponent);
```

This gives all components a UUID - which allows you to rename these structs without breaking serialization. The UUID
can be anything you like - there are online generators for UUIDs that make it easy. (Pro tip: generate a hundred or so
and save them to a file for future use!)

It is possible to use components that can't be serialized. We'll cover that later. The general approach is that you'd
define a definition that can be serialized and implement a trait to transform it into what you'd like to use at runtime.
This trait implementation can access data stored in resources and other components on the same entity.

The next step is to create a component registry:

```rust
// Register all components (based on legion_prefab::register_component_type! macro)
let component_registry = minimum::ComponentRegistryBuilder::new()
    .auto_register_components()
    .build();
```

Having the component registry isn't technically necessary yet, but we might as well do it at the same time as setting
up the components. 

## Creating the Prefab

Now starting from tutorial 1 and the above changes, we have a legion world and a component registry. To convert the
world into a prefab:

```rust
use legion_prefab::Prefab;

// Create the prefab
let prefab = Prefab::new(prefab_world);
```

Prefabs can contain Entities and EntityRefs. The EntityRef is a pointer to another prefab that indicates we should copy
entities from that prefab into this one. We won't cover EntityRefs yet - partly because it's complex, and partly because
the feature still needs more work.

Every entity that exists in a prefab gets its own UUID. It's stored in prefab.prefab_meta.entities. You can get it like
this:

```rust
// Get the UUID of the entity
let entity_uuid = prefab
    .prefab_meta
    .entities
    .iter()
    .find(|(_, value)| **value == prefab_entity)
    .map(|(entity_uuid, _)| *entity_uuid)
    .unwrap();
```

In addition, entities are placed in a legion world within the prefab. Because this world is a copy of the original world,
the Entity will be different. However, you can use the UUID to look it up.

```rust
// Look up the entity associated with the entity_uuid. To do this, we have to:
// - Look at the original prefab to find the UUID of the entity
// - Then use prefab_meta on the deserialized prefab to find the entity in the deserialized
//   prefab's world
let entity = prefab.prefab_meta.entities[&entity_uuid];

let position = prefab
    .world
    .entry_ref(entity)
    .unwrap()
    .into_component::<PositionComponent>()
    .unwrap();
println!(
    "Position of {} is {}",
    uuid::Uuid::from_bytes(entity_uuid),
    position.value
);
```

Up to this point we've created a prefab. In the next chapters, we'll go over how to save and load prefabs to disk and
how to instantiate them into a runtime world. 