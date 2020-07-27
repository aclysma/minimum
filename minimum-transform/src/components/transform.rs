use serde::{Deserialize, Serialize};
use serde_diff::SerdeDiff;
use type_uuid::TypeUuid;
use imgui_inspect_derive::Inspect;
use minimum_math::matrix::Mat4;
use legion::prelude::Entity;
use minimum_math::math::Vec3;
use glam::Quat;

//
// Primary transform component, usually populated by using other components
//
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Inspect)]
#[uuid = "35657365-bb0c-4306-8c69-d5e158ad978f"]
pub struct TransformComponentDef {
    #[serde_diff(opaque)]
    pub position: Vec3,
    // representation order: x=roll, y=pitch, z=yaw
    // rotation order: yaw, pitch, roll (zyx)
    #[serde_diff(opaque)]
    pub rotation: Vec3,
    pub scale: f32,
    #[serde_diff(opaque)]
    pub non_uniform_scale: Vec3,
}

impl Default for TransformComponentDef {
    fn default() -> Self {
        TransformComponentDef {
            position: Default::default(),
            rotation: Default::default(),
            scale: 1.0,
            non_uniform_scale: glam::Vec3::new(1.0, 1.0, 1.0).into()
        }
    }
}

// http://www.euclideanspace.com/maths/geometry/rotations/conversions/quaternionToEuler/
// original code used yaw=Y, pitch=Z, roll=X applied in that order
// this corresponds to yaw=Z, pitch=Y, roll=X applied in that order
fn quat_to_ypr(q: glam::Quat) -> (f32, f32, f32) {
    let x = q.x();
    let z = q.y();
    let y = q.z();
    let w = q.w();

    let test = x * y + z * w;
    if test >= 0.5 {
        let yaw = 2.0 * f32::atan2(x, w);
        let pitch = std::f32::consts::FRAC_PI_2;
        let roll = 0.0;

        (yaw, pitch, roll)
    } else if test <= -0.5 {
        let yaw = -2.0 * f32::atan2(x, w);
        let pitch = -std::f32::consts::FRAC_PI_2;
        let roll = 0.0;

        (yaw, pitch, roll)
    } else {
        let x_sq = x * x;
        let y_sq = y * y;
        let z_sq = z * z;
        let yaw = f32::atan2(2.0 * y * w - 2.0 * x * z, 1.0 - 2.0 * y_sq - 2.0 * z_sq);
        let pitch = f32::asin(2.0 * test);
        let roll = f32::atan2(2.0 * x * w - 2.0 * y * z, 1.0 - 2.0 * x_sq - 2.0 * z_sq);

        (yaw, pitch, roll)
    }
}

//GLTF: facing +Z, up +Y
//blender: facing -Y, up +Z
// 90deg rotation around X fixes it

impl TransformComponentDef {
    pub fn from_matrix(matrix: glam::Mat4) -> Self {
        let (scale_combined, rotation_q, position) = matrix.to_scale_rotation_translation();
        let rotation_ypr = quat_to_ypr(rotation_q);

        let rotation_xyz = (rotation_ypr.2, rotation_ypr.1, rotation_ypr.0);
        //let rotation_xyz = (rotation_ypr.2, rotation_ypr.0, rotation_ypr.1);
        //let rotation_xyz = (rotation_ypr.1, rotation_ypr.2, rotation_ypr.0);

        let scale = scale_combined.x();
        let non_uniform_scale = scale_combined / scale;

        TransformComponentDef {
            position: position.into(),
            rotation: glam::Vec3::from(rotation_xyz).into(),
            scale,
            non_uniform_scale: non_uniform_scale.into()
        }
    }

    pub fn position(&self) -> glam::Vec3 {
        *self.position
    }

    pub fn position_mut(&mut self) -> &mut glam::Vec3 {
        &mut *self.position
    }

    /// Get the world-space rotation
    pub fn rotation_quat(&self) -> glam::Quat {
        // Default is rotating around y, x, z (i.e. y is up)
        // We are z-up, so sandwich with an X axis rotation. This is temporary until I can do
        // a better rotation system
        glam::Quat::from_rotation_z(self.rotation.z()) * // yaw?
        glam::Quat::from_rotation_x(self.rotation.x()) * // pitch?
        glam::Quat::from_rotation_y(self.rotation.y())   // roll?
        // glam::Quat::from_rotation_y(self.rotation.y()) *
        //     glam::Quat::from_rotation_x(self.rotation.x()) *
        //     glam::Quat::from_rotation_z(self.rotation.z())


        //glam::Quat::from_rotation_x(std::f32::consts::FRAC_PI_2) *
            //glam::Quat::from_rotation_ypr(self.rotation.z(), self.rotation.y(), self.rotation.x()) *
            //glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
    }

    pub fn rotation_euler(&self) -> glam::Vec3 {
        *self.rotation
    }

    pub fn rotation_euler_mut(&mut self) -> &mut glam::Vec3 {
        &mut *self.rotation
    }

    /// Get the uniform scaling factor
    pub fn uniform_scale(&self) -> f32 {
        self.scale
    }

    pub fn uniform_scale_mut(&mut self) -> &mut f32 {
        &mut self.scale
    }

    pub fn non_uniform_scale(&self) -> glam::Vec3 {
        *self.non_uniform_scale
    }

    pub fn non_uniform_scale_mut (&mut self) -> &mut glam::Vec3 {
        &mut *self.non_uniform_scale
    }

    /// Get the uniform and non-uniform scale factors combined
    pub fn scale(&self) -> glam::Vec3 {
        self.scale * *self.non_uniform_scale
    }
}

legion_prefab::register_component_type!(TransformComponentDef);


#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug)]
#[uuid = "3f76404d-abdf-40dd-9bc3-997c1726d3f0"]
pub struct TransformComponent {
    // It may be possible to pack scale in the w components of each simd register later on rather
    // than having them mixed with the 3x3 rotation
    #[serde_diff(opaque)]
    pub transform: glam::Mat4
}
legion_prefab::register_component_type!(TransformComponent);

impl Default for TransformComponent {
    fn default() -> Self {
        TransformComponent {
            transform: glam::Mat4::identity()
        }
    }
}

impl TransformComponent {
    pub fn from_position(position: glam::Vec3) -> Self {
        TransformComponent {
            transform: glam::Mat4::from_translation(position)
        }
    }

    pub fn transform(&self) -> glam::Mat4 {
        self.transform
    }

    /// Get the world-space position
    pub fn position(&self) -> glam::Vec3 {
        self.transform.w_axis().truncate()
    }

    /// Get the world-space rotation
    pub fn rotation(&self) -> glam::Quat {
        self.transform.to_scale_rotation_translation().1
    }

    /// Get the uniform scaling factor
    pub fn uniform_scale(&self) -> f32 {
        let scale = self.transform.to_scale_rotation_translation().0;
        scale.x()
    }

    pub fn non_uniform_scale(&self) -> glam::Vec3 {
        let scale = self.transform.to_scale_rotation_translation().0;
        scale / scale.x()
    }

    /// Get the uniform and non-uniform scale factors combined
    pub fn scale(&self) -> glam::Vec3 {
        self.transform.to_scale_rotation_translation().0
    }

    /*
    /// Get a transform matrix that applies position/rotation/scale to move the object to
    /// world-space
    pub fn transform(&self) -> glam::Mat4 {
        unimplemented!();
    }

    /// Apply the world-space rotation to unit_x
    pub fn forward(&self) -> glam::Vec3 {
        let scale = self.transform.to_scale_rotation_translation().0;
    }

    /// Apply the world-space rotation to unit_z
    pub fn up(&self) -> glam::Vec3 {
        unimplemented!();
    }

    /// Apply the world-space rotation to unit_y
    pub fn left(&self) -> glam::Vec3 {
        unimplemented!();
    }

    /// Set the world-space position
    pub fn set_position(&mut self, offset: glam::Vec3) {

    }

    /// Apply the world-space rotation to the offset and add it to the global position
    pub fn translate_local(&mut self, offset: glam::Vec3) {

    }
    */

    pub fn set_position(&mut self, position: glam::Vec3) {
        self.transform.set_w_axis(position.extend(1.0));
    }
}

impl From<TransformComponentDef> for TransformComponent {
    fn from(from: TransformComponentDef) -> Self {
        TransformComponent {
            transform: glam::Mat4::from_scale_rotation_translation(from.scale(), from.rotation_quat(), from.position())
        }
    }
}