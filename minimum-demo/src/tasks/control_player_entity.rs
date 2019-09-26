use rendy::wsi::winit;

use crate::base::resource::{DataRequirement, Read, Write};
use crate::base::{ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::framework::resources::InputState;
use crate::framework::resources::MouseButton;
use crate::framework::resources::CameraState;
use crate::resources::PhysicsManager;
use crate::framework::resources::TimeState;

use crate::components;
use crate::base::component::ReadComponent;
use crate::base::ComponentStorage;
use crate::base::EntityFactory;

pub struct ControlPlayerEntity;
pub type ControlPlayerEntityTask = crate::base::ResourceTask<ControlPlayerEntity>;
impl ResourceTaskImpl for ControlPlayerEntity {
    type RequiredResources = (
        Read<crate::base::EntitySet>,
        Read<InputState>,
        Read<TimeState>,
        Read<CameraState>,
        ReadComponent<components::PlayerComponent>,
        ReadComponent<crate::framework::components::TransformComponent>,
        Write<EntityFactory>,
        ReadComponent<components::PhysicsBodyComponent>,
        Write<PhysicsManager>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<crate::base::task::PhasePrePhysicsGameplay>();
        config.run_only_if(crate::framework::context_flags::PLAYMODE_PLAYING);
    }

    fn run(
        //&mut self,
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            input_state,
            time_state,
            camera_state,
            player_components,
            transform_components,
            mut entity_factory,
            physics_body_components,
            mut physics_manager,
        ) = data;

        use winit::event::VirtualKeyCode;

        for (entity, _p) in player_components.iter(&entity_set) {
            if let (Some(pos), Some(physics_body_component)) = (
                transform_components.get(&entity),
                physics_body_components.get(&entity),
            ) {
                let mut direction: glm::Vec2 = glm::zero();

                if input_state.is_key_down(crate::framework::resources::KeyboardButton::new(VirtualKeyCode::S as u32)) {
                    direction.y -= 1.0;
                }

                if input_state.is_key_down(crate::framework::resources::KeyboardButton::new(VirtualKeyCode::W as u32)) {
                    direction.y += 1.0;
                }

                if input_state.is_key_down(crate::framework::resources::KeyboardButton::new(VirtualKeyCode::A as u32)) {
                    direction.x -= 1.0;
                }

                if input_state.is_key_down(crate::framework::resources::KeyboardButton::new(VirtualKeyCode::D as u32)) {
                    direction.x += 1.0;
                }

                let physics_world = physics_manager.world_mut();
                let body = physics_world
                    .rigid_body_mut(physics_body_component.body_handle())
                    .unwrap();

                #[cfg(feature = "dim3")]
                let direction = glm::vec2_to_vec3(&direction);
                body.set_velocity(nphysics::math::Velocity::new(direction * 150.0, glm::zero()));

                if input_state.is_mouse_down(MouseButton::Left) {
                    let target_position =
                        camera_state.ui_space_to_world_space(input_state.mouse_position());

                    let mut velocity = target_position - pos.position().xy();

                    if glm::magnitude(&velocity) > 0.0 {
                        velocity = glm::normalize(&velocity) * 300.0;
                    }

                    crate::constructors::create_bullet(
                        pos.position().xy(),
                        velocity,
                        &time_state,
                        &mut entity_factory,
                    );
                }
            }
        }
    }
}
