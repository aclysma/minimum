# Cooking Prefabs

*Compiling code for this demo is located in [/examples/tutorial/examples](../../examples/tutorial/examples)*

Prefabs come in two forms - cooked, and uncooked.

Uncooked prefabs can reference other prefabs. "Cooking" a prefab is the process of flattening a prefab and all the other
prefabs it depends on into a single prefab. At edit time, we want to use the hierarchy to avoid duplicated data, but at
runtime we don't want spawning to require merging a bunch of data. Cooking allows us to prepare data offline for
spawning.

The long-term goal is that this is handled for you within atelier-assets, but some work remains to be done. So currently
we cook the data at runtime right before we use it.

## Cooking a Prefab

As mentioned before, the cooking process requires having all the dependency prefabs. Again, in the future this is
expected to be handled in atelier-assets, but for now we have to do it manually.

Prefabs must be cooked in dependency order, based on overrides. For example, if prefab B
overrides something on prefab A, and prefab C overrides something on prefab B, we must cook
A, then B, then C. When cooking B, we must be able to access A in cooked form, and when
cooking C, we must be able to access B in cooked form.

In this case there are no overrides, so our ordering is simply the prefab we want to cook
and the lookup just contains this prefab.

```rust
//
// Cook the prefab world
//

// Prefabs must be cooked in dependency order, based on overrides. For example, if prefab B
// overrides something on prefab A, and prefab C overrides something on prefab B, we must cook
// A, then B, then C. When cooking B, we must be able to access A in cooked form, and when
// cooking C, we must be able to access B in cooked form.
//
// In this case there are no overrides, so our ordering is simply the prefab we want to cook
// and the lookup just contains this prefab.
//
// In the future this can be happened in atelier as part of the asset build process. For now,
// cooking happens at runtime in minimum but dependency management is automatically handled.
let prefab_cook_order = vec![prefab.prefab_id()];
let mut prefab_lookup = HashMap::new();
prefab_lookup.insert(prefab.prefab_id(), &prefab);

let cooked_prefab = legion_prefab::cook_prefab(
    component_registry.components(),
    component_registry.components_by_uuid(),
    prefab_cook_order.as_slice(),
    &prefab_lookup,
);
```

The cooked prefab contains its own legion world - a copy of the data of all upstream prefabs. This includes overridden
data, which we will cover later.

```rust
// Look up the entity associated with the entity_uuid
let cooked_entity = cooked_prefab.entities[&entity_uuid];

let position = cooked_prefab
    .world
    .entry_ref(cooked_entity)
    .unwrap()
    .into_component::<PositionComponent>()
    .unwrap();
println!(
    "Position of {} is {}",
    uuid::Uuid::from_bytes(entity_uuid),
    position.value
);
```