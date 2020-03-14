use atelier_assets::importer::{typetag, SerdeImportable};
use atelier_assets::loader::handle::Handle;
use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use imgui_inspect_derive::Inspect;
use crate::math::Vec2;

use atelier_assets::importer as atelier_importer;

//
// 2D Position
//
#[derive(
    TypeUuid, Clone, Serialize, Deserialize, SerdeImportable, SerdeDiff, Debug, Inspect, Default,
)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107194cf0"]
pub struct Position2DComponent {
    #[serde_diff(opaque)]
    pub position: Vec2,
}

legion_prefab::register_component_type!(Position2DComponent);

//
// Uniform 2D Scale
//
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeImportable, SerdeDiff, Debug, Inspect)]
#[uuid = "ea1118ac-ebbe-433b-8532-e8938cd3a2dc"]
pub struct UniformScale2DComponent {
    pub uniform_scale: f32,
}

impl Default for UniformScale2DComponent {
    fn default() -> Self {
        UniformScale2DComponent { uniform_scale: 1.0 }
    }
}

legion_prefab::register_component_type!(UniformScale2DComponent);

//
// Non-uniform 2D Scale
//
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeImportable, SerdeDiff, Debug, Inspect)]
#[uuid = "3318484f-d816-4f8e-b6d2-accd66e49276"]
pub struct NonUniformScale2DComponent {
    #[serde_diff(opaque)]
    pub non_uniform_scale: Vec2,
}

impl Default for NonUniformScale2DComponent {
    fn default() -> Self {
        NonUniformScale2DComponent {
            non_uniform_scale: glam::Vec2::new(1.0, 1.0).into(),
        }
    }
}

legion_prefab::register_component_type!(NonUniformScale2DComponent);

//
// 2D Rotation
//
#[derive(
    TypeUuid, Clone, Serialize, Deserialize, SerdeImportable, SerdeDiff, Debug, Inspect, Default,
)]
#[uuid = "6841f13d-fe38-4320-a8f8-1a6133f45e33"]
pub struct Rotation2DComponent {
    pub rotation: f32,
}

legion_prefab::register_component_type!(Rotation2DComponent);
