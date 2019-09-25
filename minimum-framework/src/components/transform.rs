#[cfg(feature = "editor")]
use crate::inspect::common_types::*;

#[cfg(feature = "editor")]
use imgui_inspect_derive::Inspect;
use minimum::component::VecComponentStorage;
use crate::CloneComponentPrototype;

#[cfg(feature = "dim2")]
pub type Position = glm::Vec2;
#[cfg(feature = "dim2")]
pub type Scale = glm::Vec2;
#[cfg(feature = "dim2")]
pub type Rotation = f32;

#[cfg(all(feature = "dim2", feature = "editor"))]
pub type ImPosition = ImGlmVec2;
#[cfg(all(feature = "dim2", feature = "editor"))]
pub type ImScale = ImGlmVec2;
#[cfg(all(feature = "dim2", feature = "editor"))]
pub type ImRotation = f32;


#[cfg(feature = "dim3")]
pub type Position = glm::Vec3;
#[cfg(feature = "dim3")]
pub type Scale = glm::Vec3;
#[cfg(feature = "dim3")]
pub type Rotation = glm::Quat;

#[cfg(all(feature = "dim3", feature = "editor"))]
pub type ImPosition = ImGlmVec3;
#[cfg(all(feature = "dim3", feature = "editor"))]
pub type ImScale = ImGlmVec3;
#[cfg(all(feature = "dim3", feature = "editor"))]
pub type ImRotation = ImGlmQuat;

pub fn default_position() -> Position {
    glm::zero()
}

#[cfg(feature = "dim2")]
pub fn default_scale() -> Scale {
    glm::vec2(1.0, 1.0)
}

#[cfg(feature = "dim3")]
pub fn default_scale() -> Scale {
    glm::vec3(1.0, 1.0, 1.0)
}

#[cfg(feature = "dim2")]
pub fn default_rotation() -> Rotation {
    0.0
}

#[cfg(feature = "dim3")]
pub fn default_rotation() -> Rotation {
    glm::quat_identity()
}

#[derive(Debug, Clone, Serialize, Deserialize, Inspect)]
pub struct TransformComponent {
    #[inspect(proxy_type = "ImPosition", on_set = "inspect_transform_updated")]
    position: Position,

    #[inspect(proxy_type = "ImScale", on_set = "inspect_transform_updated")]
    scale: Scale,

    #[inspect(proxy_type = "ImRotation", on_set = "inspect_transform_updated")]
    rotation: Rotation,

    #[inspect(skip)]
    #[serde(skip)]
    requires_sync_to_physics: bool,
}

pub type TransformComponentPrototype = CloneComponentPrototype<TransformComponent>;

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            position: default_position(),
            scale: default_scale(),
            rotation: default_rotation(),
            requires_sync_to_physics: false,
        }
    }
}

impl TransformComponent {
    pub fn new(position: Position, scale: Scale, rotation: Rotation) -> Self {
        TransformComponent {
            position,
            scale,
            rotation,
            requires_sync_to_physics: false,
        }
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    pub fn scale(&self) -> Scale {
        self.scale
    }

    pub fn scale_mut(&mut self) -> &mut Scale {
        &mut self.scale
    }

    pub fn uniform_scale(&self) -> f32 { self.scale.x.max(self.scale.y) }

    pub fn rotation(&self) -> Rotation { self.rotation }

    pub fn rotation_mut(&mut self) -> &mut Rotation { &mut self.rotation }

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
