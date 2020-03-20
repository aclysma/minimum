use atelier_assets::importer::{ImportedAsset, Importer, ImporterValue, SourceFileImporter};
use atelier_assets::core::AssetUuid;
use serde::{Deserialize, Serialize};
use std::io::Read;
use type_uuid::TypeUuid;

use crate::pipeline::PrefabAsset;

use legion::prelude::*;
use legion_prefab::ComponentRegistration;
use std::collections::HashMap;
use prefab_format::ComponentTypeUuid;

#[derive(Default, Deserialize, Serialize, TypeUuid, Clone, Copy)]
#[uuid = "80583980-24d4-4034-8394-ea749b43f55d"]
pub struct PrefabImporterOptions {}

/// A simple state for Importer to retain the same UUID between imports
/// for all single-asset source files
#[derive(Default, Deserialize, Serialize, TypeUuid)]
#[uuid = "14d89614-7e10-4f59-952f-af32c73bda90"]
pub struct PrefabImporterState {
    pub id: Option<AssetUuid>,
}

#[derive(Default, TypeUuid)]
#[uuid = "5bdf4d06-a1cb-437b-b182-d6d8cb23512c"]
pub struct PrefabImporter {}

use atelier_assets::importer as atelier_importer;

impl Importer for PrefabImporter {
    type State = PrefabImporterState;
    type Options = PrefabImporterOptions;

    fn version_static() -> u32 {
        1
    }

    fn version(&self) -> u32 {
        Self::version_static()
    }

    fn import(
        &self,
        source: &mut dyn Read,
        _: Self::Options,
        state: &mut Self::State,
    ) -> atelier_importer::Result<ImporterValue> {
        ///////////////////////////////////////////////////////////////
        // STEP 1: Read in the data
        ///////////////////////////////////////////////////////////////

        // Read in the data
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;

        ///////////////////////////////////////////////////////////////
        // STEP 2: Deserialize the prefab into a legion world
        ///////////////////////////////////////////////////////////////

        // Create a deserializer
        let mut de = ron::de::Deserializer::from_bytes(bytes.as_slice()).unwrap();

        // Create the component registry
        let registered_components = {
            let comp_registrations = legion_prefab::iter_component_registrations();
            use std::iter::FromIterator;
            let component_types: HashMap<ComponentTypeUuid, ComponentRegistration> =
                HashMap::from_iter(comp_registrations.map(|reg| (reg.uuid().clone(), reg.clone())));

            component_types
        };

        let prefab_serde_context = legion_prefab::PrefabSerdeContext {
            registered_components: &registered_components,
        };

        let prefab_deser = legion_prefab::PrefabFormatDeserializer::new(prefab_serde_context);
        prefab_format::deserialize(&mut de, &prefab_deser)?;
        let prefab = prefab_deser.prefab();

        let prefab_asset = PrefabAsset { prefab };

        ///////////////////////////////////////////////////////////////
        // STEP 3: Now we need to save it into an asset
        ///////////////////////////////////////////////////////////////

        {
            // Print for debug
            let legion_world_str =
                ron::ser::to_string_pretty(&prefab_asset, ron::ser::PrettyConfig::default())
                    .unwrap();

            log::trace!("Serialized legion world:");
            log::trace!("legion_world_str {}", legion_world_str);

            let mut ron_ser =
                ron::ser::Serializer::new(Some(ron::ser::PrettyConfig::default()), true);
            let prefab_ser = legion_prefab::PrefabFormatSerializer::new(
                prefab_serde_context,
                &prefab_asset.prefab,
            );
            prefab_format::serialize(&mut ron_ser, &prefab_ser, prefab_asset.prefab.prefab_id())
                .expect("failed to round-trip prefab");
            log::trace!(
                "Round-tripped legion world: {}",
                ron_ser.into_output_string()
            );
        }

        // Add the ID to the .meta
        let prefab_id = prefab_asset.prefab.prefab_id();
        state.id = Some(AssetUuid(prefab_id));

        Ok(ImporterValue {
            assets: vec![ImportedAsset {
                id: state.id.expect("AssetUuid not generated"),
                search_tags: Vec::new(),
                build_deps: Vec::new(),
                load_deps: Vec::new(),
                asset_data: Box::new(prefab_asset),
                build_pipeline: None,
            }],
        })
    }
}

inventory::submit!(SourceFileImporter {
    extension: ".prefab",
    instantiator: || Box::new(PrefabImporter::default())
});
