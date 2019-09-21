#[cfg(feature = "editor")]
use framework::inspect::common_types::*;

#[cfg(feature = "editor")]
use imgui_inspect_derive::Inspect;
use minimum::component::VecComponentStorage;
use framework::CloneComponentPrototype;


#[derive(Debug, Clone, Serialize, Deserialize, Inspect)]
pub struct TransformComponent {
    #[inspect(proxy_type = "ImGlmVec2", on_set = "inspect_transform_updated")]
    position: glm::Vec2,

    #[inspect(proxy_type = "ImGlmVec2", on_set = "inspect_transform_updated")]
    scale: glm::Vec2,

    #[inspect(on_set = "inspect_transform_updated")]
    rotation: f32,

    #[inspect(skip)]
    #[serde(skip)]
    requires_sync_to_physics: bool,
}

pub type TransformComponentPrototype = CloneComponentPrototype<TransformComponent>;

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            position: glm::zero(),
            scale: glm::zero(),
            rotation: 0.0,
            requires_sync_to_physics: false,
        }
    }
}

impl TransformComponent {
    pub fn new(position: glm::Vec2, scale: glm::Vec2, rotation: f32) -> Self {
        TransformComponent {
            position,
            scale,
            rotation,
            requires_sync_to_physics: false,
        }
    }

    pub fn position(&self) -> glm::Vec2 {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut glm::Vec2 {
        &mut self.position
    }

    pub fn scale(&self) -> glm::Vec2 {
        self.scale
    }

    pub fn scale_mut(&mut self) -> &mut glm::Vec2 {
        &mut self.scale
    }

    pub fn uniform_scale(&self) -> f32 { self.scale.x.max(self.scale.y) }

    pub fn rotation(&self) -> f32 { self.rotation }

    pub fn rotation_mut(&mut self) -> &mut f32 { &mut self.rotation }

    pub fn requires_sync_to_physics(&self) -> bool {
        self.requires_sync_to_physics
    }

    pub fn clear_requires_sync_to_physics(&mut self) {
        self.requires_sync_to_physics = false
    }

    //TODO: Replace with a better solution that doesn't require O(n) iteration
    // - Allow register callback on inspector?
    // - Some sort of flagging system?
    // - Attach an extra component?
    pub fn inspect_transform_updated(&mut self) {
        self.requires_sync_to_physics = true;
    }

    pub fn editor_transform_updated(&mut self) {
        self.requires_sync_to_physics = true;
    }
}

impl minimum::Component for TransformComponent {
    type Storage = VecComponentStorage<Self>;
}
