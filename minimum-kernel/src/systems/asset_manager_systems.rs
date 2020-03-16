use legion::prelude::*;
use crate::resources::AssetResource;

pub fn update_asset_manager() -> Box<dyn Schedulable> {
    SystemBuilder::new("update asset manager")
        .write_resource::<AssetResource>()
        .build(|_, _, asset_manager, _| {
            asset_manager.update();
        })
}
