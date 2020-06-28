use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use imgui_inspect_derive::Inspect;
use minimum_math::math::Vec3;

//
// Uniform 2D Scale
//
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Inspect)]
#[uuid = "ea1118ac-ebbe-433b-8532-e8938cd3a2dc"]
pub struct UniformScaleComponent {
    pub uniform_scale: f32,
}

impl Default for UniformScaleComponent {
    fn default() -> Self {
        UniformScaleComponent { uniform_scale: 1.0 }
    }
}

legion_prefab::register_component_type!(UniformScaleComponent);

//
// Non-uniform 2D Scale
//
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Inspect)]
#[uuid = "3318484f-d816-4f8e-b6d2-accd66e49276"]
pub struct NonUniformScaleComponent {
    #[serde_diff(opaque)]
    pub non_uniform_scale: Vec3,
}

impl Default for NonUniformScaleComponent {
    fn default() -> Self {
        NonUniformScaleComponent {
            non_uniform_scale: glam::Vec3::new(1.0, 1.0, 1.0).into(),
        }
    }
}

legion_prefab::register_component_type!(NonUniformScaleComponent);
