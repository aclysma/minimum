
pub mod common_types;
mod entity_inspector;

pub use entity_inspector::InspectorComponentRegistry;

//use imgui_inspect_derive::Inspect;
/*
#[derive(Inspect, Clone)]
pub struct MyStruct {
    pub a: f32,
    pub b: f32,
    #[inspect(wrapping_type = "ImGlmVec2")]
    pub c: glm::Vec2,
    #[inspect(wrapping_type = "ImGlmVec3")]
    pub d: glm::Vec3
}

#[derive(Inspect)]
pub struct MyStruct2 {

    #[inspect(min_value = 5.0, max_value = 42.0)]
    pub a: f32,
    #[inspect_slider(min_value = 5.0, max_value = 53.0)]
    pub b: f32,
    #[inspect(wrapping_type = "ImGlmVec2")]
    pub c: glm::Vec2,
    #[inspect(wrapping_type = "ImGlmVec3", min_value = 100.0)]
    pub d: glm::Vec3,

    pub s: MyStruct
}

*/