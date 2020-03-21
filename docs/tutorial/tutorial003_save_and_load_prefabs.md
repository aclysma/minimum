# Saving and Loading Prefabs

Starting from tutorial 2, we have set up components and a component registry. We will also use the same legion world.

First lets convert that world to a prefab

```rust
// Create the prefab
let prefab_uncooked = Prefab::new(prefab_world);

// Get the UUID of the entity. This UUID is maintained throughout the whole chain.
let entity_uuid = prefab_uncooked
    .prefab_meta
    .entities
    .iter()
    .find(|(_, value)| **value == prefab_entity)
    .map(|(entity_uuid, _)| *entity_uuid)
    .unwrap();
```

## Serializing the Prefab

Now we can serialize it to a string. I've wrapped the logic in a function since there's a few moving parts:
 * prefab_serde_context - Provides the component lookup to the serialization code
 * ron_ser - Responsible for writing out the data in RON format
 * prefab_ser - Holds references to everything the serializer needs
 * prefab_format::serialize - Does the actual work to serialize the data

After prefab_format::serialize is called, the RON serializer holds the produced string

```rust
fn serialize_prefab(
    component_registry: &ComponentRegistry,
    prefab: &Prefab,
) -> String {
    let prefab_serde_context = legion_prefab::PrefabSerdeContext {
        registered_components: component_registry.components_by_uuid(),
    };

    let mut ron_ser = ron::ser::Serializer::new(Some(ron::ser::PrettyConfig::default()), true);
    let prefab_ser = legion_prefab::PrefabFormatSerializer::new(prefab_serde_context, prefab);

    prefab_format::serialize(&mut ron_ser, &prefab_ser, prefab.prefab_id())
        .expect("failed to round-trip prefab");

    ron_ser.into_output_string()
}

// Serialize the prefab to a string
let serialized_prefab = serialize_prefab(&component_registry, &prefab_uncooked);
```

## Prefab Format

It looks something like this:

```rust
Prefab(
    id: "599a55e1-1e07-4604-98a8-7df40ae9ee78",
    objects: [
        Entity(PrefabEntity(
            id: "25a63c38-f7ab-4e44-8f59-ef2f05054e94",
            components: [
                EntityComponent(
                    type: "8bf67228-f96c-4649-b306-ecd107190000",
                    data: PositionComponent(
                        value: Vec2(0, 500),
                    ),
                ),
                EntityComponent(
                    type: "8bf67228-f96c-4649-b306-ecd107190001",
                    data: VelocityComponent(
                        value: Vec2(5, 0),
                    ),
                ),
            ],
        )),
    ],
)
```

I'd like to point out a few pieces:
 * The prefab gets assigned a random UUID automatically
 * Each entity in the world also gets a UUID
 * Each entity can have components. The type is identified by the UUID specified in
   code when decorating the component type
 * The raw data is defined by serde

## Deserializing the Prefab

We can now read it back from a string (or a file.) The process is a mirror of serializing.

```rust
fn deserialize_prefab(
    component_registry: &ComponentRegistry,
    serialized: &str,
) -> Prefab {
    let prefab_serde_context = legion_prefab::PrefabSerdeContext {
        registered_components: component_registry.components_by_uuid(),
    };

    let mut deserializer = ron::de::Deserializer::from_str(serialized).unwrap();

    let prefab_deser = legion_prefab::PrefabFormatDeserializer::new(prefab_serde_context);
    prefab_format::deserialize(&mut deserializer, &prefab_deser).unwrap();
    prefab_deser.prefab()
}

// Deserialize the world from a string
let deserialized_prefab = deserialize_prefab(&component_registry, &serialized_prefab);
```

If you use atelier-assets and minimum, you won't need to work with files or serialization directly,
but it's still nice to know it's an option!

We'll end this chapter as we did the prior chapter - show how to access the data in the deserialized
prefab by UUID. (Again, the deserialized prefab's world is different and you can't use the same entity
as before.)

```rust
// Look up the entity associated with the entity_uuid. To do this, we have to:
// - Look at the original prefab to find the UUID of the entity
// - Then use prefab_meta on the deserialized prefab to find the entity in the deserialized
//   prefab's world
let mut deserialized_entity = deserialized_prefab.prefab_meta.entities[&entity_uuid];

// Now run some code to demonstrate that we found the same entity in the deserialized world and
// that we get the same results as before
let position = deserialized_prefab
    .world
    .get_component::<PositionComponent>(deserialized_entity)
    .unwrap();

println!(
    "Position of {} is {}",
    uuid::Uuid::from_bytes(entity_uuid),
    position.value
);
```