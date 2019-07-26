use minimum::systems::{async_dispatch::Task, DataRequirement, Read, Write};

use crate::resources::{InputManager, MouseButtons, RenderState, TimeState};

use crate::components;
use minimum::component::{ReadComponent, WriteComponent};
use minimum::ComponentStorage;

mod imgui_tasks;
pub use imgui_tasks::ImguiBeginFrame;
pub use imgui_tasks::RenderImguiMainMenu;

mod debug_draw_tasks;
pub use debug_draw_tasks::UpdateDebugDraw;

mod input_manager_tasks;
pub use input_manager_tasks::GatherInput;

mod physics_tasks;
pub use physics_tasks::UpdatePhysics;
pub use physics_tasks::UpdatePositionFromPhysics;

pub struct UpdateTimeState;
impl Task for UpdateTimeState {
    type RequiredResources = (Write<TimeState>);

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let mut time_state = data;
        time_state.update();
    }
}

pub struct ControlPlayerEntity;
impl Task for ControlPlayerEntity {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<InputManager>,
        Read<TimeState>,
        Read<RenderState>,
        ReadComponent<components::PlayerComponent>,
        WriteComponent<components::PositionComponent>,
        Write<crate::constructors::BulletFactory>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            entity_set,
            input_manager,
            time_state,
            render_state,
            player_components,
            mut position_components,
            mut bullet_factory,
        ) = data;

        let dt = time_state.previous_frame_dt;

        use winit::event::VirtualKeyCode;

        for (entity, _p) in player_components.iter(&entity_set) {
            if let Some(pos) = position_components.get_mut(&entity) {
                if input_manager.is_key_down(VirtualKeyCode::S) {
                    pos.position_mut().y -= dt * 150.0;
                }

                if input_manager.is_key_down(VirtualKeyCode::W) {
                    pos.position_mut().y += dt * 150.0;
                }

                if input_manager.is_key_down(VirtualKeyCode::A) {
                    pos.position_mut().x -= dt * 150.0;
                }

                if input_manager.is_key_down(VirtualKeyCode::D) {
                    pos.position_mut().x += dt * 150.0;
                }

                if input_manager.is_mouse_down(MouseButtons::Left) {
                    let target_position =
                        render_state.ui_space_to_world_space(input_manager.mouse_position());

                    let mut velocity = target_position - pos.position();

                    if glm::magnitude(&velocity) > 0.0 {
                        velocity = glm::normalize(&velocity) * 300.0;
                    }

                    bullet_factory.enqueue_create(crate::constructors::BulletPrototype::new(
                        pos.position(),
                        velocity,
                    ));
                }
            }
        }
    }
}

pub struct UpdatePositionWithVelocity;
impl Task for UpdatePositionWithVelocity {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<TimeState>,
        WriteComponent<components::PositionComponent>,
        ReadComponent<components::VelocityComponent>,
        ReadComponent<components::PhysicsBodyComponent>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            entity_set,
            time_state,
            mut position_components,
            velocity_components,
            physics_body_components,
        ) = data;

        let dt = time_state.previous_frame_dt;

        for (entity, vel) in velocity_components.iter(&entity_set) {
            if physics_body_components.exists(&entity) {
                // Skip any entities that have a physics body as movement is being controlled by
                // nphysics
                continue;
            }

            if let Some(pos) = position_components.get_mut(&entity) {
                *pos.position_mut() += vel.velocity() * dt;
            }
        }
    }
}

pub struct HandleFreeAtTimeComponents;
impl Task for HandleFreeAtTimeComponents {
    type RequiredResources = (
        Write<minimum::EntitySet>,
        Read<TimeState>,
        ReadComponent<components::FreeAtTimeComponent>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (mut entity_set, time_state, free_at_time_components) = data;

        //TODO-API: Find a better way to do this.. deferred delete is fine
        let mut entities_to_free = vec![];
        for (entity, free_at_time) in free_at_time_components.iter(&entity_set) {
            if free_at_time.should_free(&time_state) {
                entities_to_free.push(entity);
            }
        }

        for e in entities_to_free {
            entity_set.enqueue_free(&e);
        }
    }
}
