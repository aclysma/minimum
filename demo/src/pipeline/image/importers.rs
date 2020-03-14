use std::io::Read;

use atelier_assets::core::AssetUuid;
use atelier_assets::importer::{Error, ImportedAsset, Importer, ImporterValue, Result};
use serde::{Deserialize, Serialize};
use type_uuid::*;

use crate::pipeline::image::*;

#[derive(TypeUuid, Serialize, Deserialize, Default)]
#[uuid = "3c8367c8-45fb-40bb-a229-00e5e9c3fc70"]
struct SimpleState(Option<AssetUuid>);

#[derive(TypeUuid)]
#[uuid = "720d636b-b79c-42d4-8f46-a2d8e1ada46e"]
struct ImageImporter;
impl Importer for ImageImporter {
    fn version_static() -> u32
    where
        Self: Sized,
    {
        1
    }
    fn version(&self) -> u32 {
        Self::version_static()
    }

    type Options = ();

    type State = SimpleState;

    /// Reads the given bytes and produces assets.
    fn import(
        &self,
        source: &mut dyn Read,
        _options: Self::Options,
        state: &mut Self::State,
    ) -> Result<ImporterValue> {
        let id = state
            .0
            .unwrap_or_else(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));
        *state = SimpleState(Some(id));
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;
        let asset =
            ImageAsset::Rgb8(image2::io::decode(&bytes).map_err(|e| Error::Boxed(Box::new(e)))?);
        Ok(ImporterValue {
            assets: vec![ImportedAsset {
                id,
                search_tags: vec![],
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(asset),
            }],
        })
    }
}
// make a macro to reduce duplication here :)
inventory::submit!(atelier_assets::importer::SourceFileImporter {
    extension: "png",
    instantiator: || Box::new(ImageImporter {}),
});
inventory::submit!(atelier_assets::importer::SourceFileImporter {
    extension: "jpg",
    instantiator: || Box::new(ImageImporter {}),
});
inventory::submit!(atelier_assets::importer::SourceFileImporter {
    extension: "tga",
    instantiator: || Box::new(ImageImporter {}),
});
