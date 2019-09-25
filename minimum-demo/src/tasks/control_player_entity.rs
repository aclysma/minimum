use rendy::wsi::winit;

use minimum::resource::{DataRequirement, Read, Write};
use minimum::{ResourceTaskImpl, TaskConfig, TaskContextFlags};

use crate::resources::{InputManager, MouseButtons, PhysicsManager, RenderState};
use framework::resources::TimeState;

use crate::components;
use minimum::component::ReadComponent;
use minimum::ComponentStorage;
use minimum::EntityFactory;

pub struct ControlPlayerEntity;
pub type ControlPlayerEntityTask = minimum::ResourceTask<ControlPlayerEntity>;
impl ResourceTaskImpl for ControlPlayerEntity {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<InputManager>,
        Read<TimeState>,
        Read<RenderState>,
        ReadComponent<components::PlayerComponent>,
        ReadComponent<framework::components::TransformComponent>,
        Write<EntityFactory>,
        ReadComponent<components::PhysicsBodyComponent>,
        Write<PhysicsManager>,
    );

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<minimum::task::PhasePrePhysicsGameplay>();
        config.run_only_if(framework::context_flags::PLAYMODE_PLAYING);
    }

    fn run(
        //&mut self,
        _context_flags: &TaskContextFlags,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            input_manager,
            time_state,
            render_state,
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

                if input_manager.is_key_down(VirtualKeyCode::S) {
                    direction.y -= 1.0;
                }

                if input_manager.is_key_down(VirtualKeyCode::W) {
                    direction.y += 1.0;
                }

                if input_manager.is_key_down(VirtualKeyCode::A) {
                    direction.x -= 1.0;
                }

                if input_manager.is_key_down(VirtualKeyCode::D) {
                    direction.x += 1.0;
                }

                let physics_world = physics_manager.world_mut();
                let body = physics_world
                    .rigid_body_mut(physics_body_component.body_handle())
                    .unwrap();

                #[cfg(feature = "dim3")]
                let direction = glm::vec2_to_vec3(&direction);
                body.set_velocity(nphysics::math::Velocity::new(direction * 150.0, glm::zero()));

                if input_manager.is_mouse_down(MouseButtons::Left) {
                    let target_position =
                        render_state.ui_space_to_world_space(input_manager.mouse_position());

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
