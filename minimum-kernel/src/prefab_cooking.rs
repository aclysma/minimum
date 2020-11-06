use crate::resources::AssetResource;

use atelier_assets::loader::{handle::Handle, storage::LoadStatus};
use std::collections::HashMap;

use legion::storage::ComponentTypeId;
use prefab_format::{ComponentTypeUuid, PrefabUuid};
use legion_prefab::{ComponentRegistration, CookedPrefab};
use crate::pipeline::PrefabAsset;
use atelier_assets::core::AssetUuid;
use std::hash::BuildHasher;

pub fn cook_prefab<F: Fn(&mut AssetResource), S: BuildHasher, T: BuildHasher>(
    asset_manager: &mut AssetResource,
    registered_components: &HashMap<ComponentTypeId, ComponentRegistration, S>,
    registered_components_by_uuid: &HashMap<ComponentTypeUuid, ComponentRegistration, T>,
    prefab_uuid: AssetUuid,
    update_fn: &F,
) -> CookedPrefab {
    // This will allow us to look up prefab handles by AssetUuid
    let mut prefab_handle_lookup = HashMap::new();

    // This will hold the asset IDs sorted with dependencies first. This ensures that
    // prefab_lookup and entity_lookup are populated with all dependent prefabs/entities
    let mut prefab_cook_order = vec![];

    // Recursively do a blocking load on the prefab and the other prefabs it depends on. This
    // populates prefab_handle_lookup and prefab_cook_order
    request_prefab_dependencies(
        asset_manager,
        prefab_uuid,
        &mut prefab_handle_lookup,
        &mut prefab_cook_order,
        update_fn,
    );

    // This will allowus to look up prefab references by AssetUuid
    let mut prefab_lookup = HashMap::new();

    for prefab_handle in prefab_handle_lookup.values() {
        let prefab_asset: &PrefabAsset = asset_manager.asset(&prefab_handle).unwrap();
        prefab_lookup.insert(prefab_asset.prefab.prefab_meta.id, &prefab_asset.prefab);
    }

    legion_prefab::cook_prefab(
        registered_components,
        registered_components_by_uuid,
        prefab_cook_order.as_slice(),
        &prefab_lookup,
    )
}

// This function does a recursive blocking load on the provided prefab asset and all prefabs
// that it references. As it does this, prefab_lookup and prefab_cook_order are populated
fn request_prefab_dependencies<F: Fn(&mut AssetResource)>(
    asset_manager: &mut AssetResource,
    id: AssetUuid,
    prefab_lookup: &mut HashMap<PrefabUuid, Handle<PrefabAsset>>,
    prefab_cook_order: &mut Vec<PrefabUuid>,
    update_fn: &F,
) {
    // Request the asset
    let handle = asset_manager.load_asset::<PrefabAsset>(id);

    // Block until it loads
    loop {
        (update_fn)(asset_manager);
        if let LoadStatus::Loaded = asset_manager.load_status(&handle) {
            break;
        }
    }

    // Grab a reference to the asset
    let prefab_asset: &PrefabAsset = asset_manager.asset(&handle).unwrap();

    // Get a list of prefabs this asset references. We clone these into a new list due to borrowing restrictions
    let other_prefab_ids: Vec<_> = prefab_asset
        .prefab
        .prefab_meta
        .prefab_refs
        .iter()
        .map(|(other_prefab_id, _)| AssetUuid(*other_prefab_id))
        .collect();

    // Use recursion to visit the tree ensuring that ancestor prefab data gets processed first
    for other_prefab_id in other_prefab_ids {
        if !prefab_lookup.contains_key(&other_prefab_id.0) {
            request_prefab_dependencies(
                asset_manager,
                other_prefab_id,
                prefab_lookup,
                prefab_cook_order,
                update_fn,
            );
        }
    }

    // Write data.. this needs to happen after we visit prefabs that we reference
    prefab_lookup.insert(id.0, handle);
    prefab_cook_order.push(id.0);
}
