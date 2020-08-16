use legion::*;
use crate::resources::AssetResource;

pub fn update_asset_manager(
    _world: &mut World,
    resources: &mut Resources,
) {
    resources.get_mut::<AssetResource>().unwrap().update(resources);
}
