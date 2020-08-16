# Creating Prefab Overrides

Currently we have some very basic support for prefabs. This chapter will explain how to produce a prefab that overrides
another prefab and how to cook it.

Unlike transactions, building prefabs has a more limited feature set. We hope to expand on it soon. Currently, we only
support modifying and adding components, and adding entities. 

## Producing the Prefab

```rust
// Use a prefab builder to make a new prefab that overrides a field on the given prefab
let mut prefab_builder = PrefabBuilder::new(
    prefab.prefab_id(),
    cooked_prefab,
    &component_registry.copy_clone_impl(),
);

// Here, we modify the world on the prefab builder. 
// The changes here are scanned to produce the prefab.
let prefab_builder_entity = prefab_builder.uuid_to_entity(entity_uuid).unwrap();
prefab_builder
    .world_mut()
    .get_component_mut::<PositionComponent>(prefab_builder_entity)
    .unwrap()
    .value = glam::Vec2::new(0.0, 1000.0);

// Produce the prefab that overrides the original prefab
let prefab_with_override = prefab_builder
    .create_prefab(
        component_registry.components_by_uuid(),
        &component_registry.copy_clone_impl(),
    )
    .unwrap();
```

## Cooking the Prefab

We cook the prefab as we did before except now we need to provide both prefabs to `legion_prefab::cook_prefab`.
In the long term, this process will be handled by `atelier-assets` but for now must be
done manually.

The cooked prefab can be persisted or used directly in memory.

```rust
//
// Cook the prefab that has the override
//
let prefab_cook_order = vec![prefab.prefab_id(), prefab_with_override.prefab_id()];
let mut prefab_lookup = HashMap::new();
prefab_lookup.insert(prefab.prefab_id(), &prefab);
prefab_lookup.insert(prefab_with_override.prefab_id(), &prefab_with_override);

let cooked_prefab_with_override = legion_prefab::cook_prefab(
    component_registry.components(),
    component_registry.components_by_uuid(),
    prefab_cook_order.as_slice(),
    &prefab_lookup,
);
```

Now that we have the prefab, we can look up the data in the new prefab to see that the change has taken effect.

By using prefabs in this way, any changes other than the field we just overrode will propagage into the cooked prefab.

```rust

// Look up the entity in the cooked prefab with override by UUID
let entity = cooked_prefab_with_override.entities[&entity_uuid];

let position = cooked_prefab_with_override
    .world
    .get_component::<PositionComponent>(entity)
    .unwrap();
println!(
    "Position of {} is {}",
    uuid::Uuid::from_bytes(entity_uuid),
    position.value
);
```