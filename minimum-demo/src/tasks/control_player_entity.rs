use minimum::systems::{DataRequirement, Read, Write};
use minimum::{Task, TaskContext};

use crate::resources::{InputManager, MouseButtons, PhysicsManager, RenderState, TimeState};

use crate::components;
use minimum::component::ReadComponent;
use minimum::ComponentStorage;
use minimum::EntityFactory;

#[derive(typename::TypeName)]
pub struct ControlPlayerEntity;
impl Task for ControlPlayerEntity {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<InputManager>,
        Read<TimeState>,
        Read<RenderState>,
        ReadComponent<components::PlayerComponent>,
        ReadComponent<components::PositionComponent>,
        Write<EntityFactory>,
        ReadComponent<components::PhysicsBodyComponent>,
        Write<PhysicsManager>,
    );
    const REQUIRED_FLAGS: usize = crate::context_flags::PLAYMODE_PLAYING;

    fn run(
        &mut self,
        _task_context: &TaskContext,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (
            entity_set,
            input_manager,
            time_state,
            render_state,
            player_components,
            position_components,
            mut entity_factory,
            physics_body_components,
            mut physics_manager,
        ) = data;

        use winit::event::VirtualKeyCode;

        for (entity, _p) in player_components.iter(&entity_set) {
            if let (Some(pos), Some(physics_body_component)) = (
                position_components.get(&entity),
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

                body.set_velocity(nphysics2d::math::Velocity::new(direction * 150.0, 0.0));

                if input_manager.is_mouse_down(MouseButtons::Left) {
                    let target_position =
                        render_state.ui_space_to_world_space(input_manager.mouse_position());

                    let mut velocity = target_position - pos.position();

                    if glm::magnitude(&velocity) > 0.0 {
                        velocity = glm::normalize(&velocity) * 300.0;
                    }

                    crate::constructors::create_bullet(
                        pos.position(),
                        velocity,
                        &time_state,
                        &mut entity_factory,
                    );
                }
            }
        }
    }
}
