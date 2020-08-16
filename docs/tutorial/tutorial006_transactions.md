# Transactions

Once you have a cooked prefab, you can modify it by using transactions. We use transactions for:

 * Applying changes to the runtime state
 * Sending the changes back to the atelier daemon to commit changes to disk, and possibly other connected devices
 * Implementing Undo/Redo

Using a transaction is simple:

1. Open the transaction
2. Modify the transaction's world (just as if you weren't using transactions)
3. Produce diffs from the transaction

One you have a transaction diff, you can either apply or revert the change to the prefab it is based on.

## Opening the Transaction

When opening the transaction, you must indicate what entities you intend to modify. These entities will be copied into
the transaction's world. In a typical case if you were implementing an editor, most of the time you'd include every
entity that is selected in the editor.

```rust
// Start a new transaction
let mut transaction = legion_transaction::TransactionBuilder::new()
    .add_entity(cooked_entity, entity_uuid)
    .begin(
        &cooked_prefab.world,
        &component_registry.copy_clone_impl(),
    );
```

## Modifying the Transaction's World

To modify the transaction, you can access it's world:

```rust
// Mess with a value in the transaction's world
let transaction_entity = transaction.uuid_to_entity(entity_uuid).unwrap();
transaction
    .world_mut()
    .get_component_mut::<PositionComponent>(transaction_entity)
    .unwrap()
    .value += glam::Vec2::new(0.0, 1000.0);
```

The following operations are supported:
 * Adding entities
 * Adding components
 * Changing existing components
 * Removing entities
 * Removing components

In short, whatever you do on the world will be reflected in the diffs that the transaction produces.

## Producing Diffs from the Transaction

To produce the diffs, call `create_transaction_diffs`

```rust
let diffs = transaction.create_transaction_diffs(component_registry.components_by_uuid());
```

You can use the resulting value's `apply_diff()` or `revert_diff()` to apply or undo the change.

### Applying to Prefabs

Currently, this only works with prefabs that have no overrides. We hope to implement support for this in the future.

```rust
    // Apply the change to the prefab
    // The return value is a result that may indicate failure if there are prefab overrides
    let mut prefab = legion_transaction::apply_diff_to_prefab(
        &mut prefab,
        diffs.apply_diff(),
        component_registry.components_by_uuid(),
        &component_registry.copy_clone_impl(),
    )
    .unwrap();
```

### Applying to Cooked Prefabs

```rust
    // Apply the change to the prefab
    let mut cooked_prefab = legion_transaction::apply_diff_to_cooked_prefab(
        &mut cooked_prefab,
        diffs.apply_diff(),
        component_registry.components_by_uuid(),
        &component_registry.copy_clone_impl(),
    );
```