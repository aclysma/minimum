use legion::prelude::*;
use crate::resources::AssetResource;

pub fn update_asset_manager(
    world: &mut World,
    resources: &mut Resources,
) {
    resources.get_mut::<AssetResource>().unwrap().update(resources);
}