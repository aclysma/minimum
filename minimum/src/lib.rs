pub mod daemon;

pub mod components {
    pub use minimum_transform::components::*;
}

pub mod pipeline {
    pub use minimum_kernel::pipeline::*;
}

pub mod resources {
    pub use minimum_kernel::resources::*;
    pub use minimum_game::resources::*;

    pub mod editor {
        pub use minimum_editor::resources::*;
    }
}
pub mod systems {
    pub use minimum_kernel::systems::*;
    pub use minimum_game::systems::*;

    mod editor {
        pub use minimum_editor::systems::*;
    }
}

pub use minimum_kernel::prefab_cooking;
pub use minimum_kernel::ComponentRegistry;
pub use minimum_kernel::ComponentRegistryBuilder;
pub use minimum_kernel::DynAssetLoader;
pub use minimum_kernel::UpdateAssetResult;
pub use minimum_kernel::AssetStorageSet;

pub mod util {
    pub use minimum_kernel::util::*;
}

pub use minimum_math as math;
pub use minimum_transform as transform;
pub use minimum_kernel as kernel;
pub use minimum_game as game;
pub use minimum_editor as editor;

pub use minimum_game::ImguiManager;

pub use minimum_game::input;
