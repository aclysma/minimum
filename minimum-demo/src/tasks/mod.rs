use minimum::systems::{async_dispatch::Task, DataRequirement, Read, Write};

use crate::resources::{InputManager, MouseButtons, PhysicsManager, RenderState, TimeState};

use crate::components;
use minimum::component::{ReadComponent, WriteComponent};
use minimum::ComponentStorage;
use minimum::EntityFactory;

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

#[derive(typename::TypeName)]
pub struct UpdateTimeState;
impl Task for UpdateTimeState {
    type RequiredResources = (Write<TimeState>, Read<InputManager>);

    fn run(
        &mut self,
        data: <Self::RequiredResources as DataRequirement>::Borrow,
    ) {
        let (mut time_state, input_manager) = data;

        let mut play_mode = time_state.play_mode;

        use winit::event::VirtualKeyCode;
        use crate::resources::PlayMode;
        if input_manager.is_key_just_down(VirtualKeyCode::Space) {
            play_mode = match play_mode {
                PlayMode::System => PlayMode::Playing,
                PlayMode::Paused => PlayMode::Playing,
                PlayMode::Playing => PlayMode::System,
            }
        }

        time_state.update(play_mode);
    }
}

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

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
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

#[derive(typename::TypeName)]
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

        let dt = time_state.playing().previous_frame_dt;

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

#[derive(typename::TypeName)]
pub struct HandleFreeAtTimeComponents;
impl Task for HandleFreeAtTimeComponents {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        WriteComponent<minimum::PendingDeleteComponent>,
        Read<TimeState>,
        ReadComponent<components::FreeAtTimeComponent>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (entity_set, mut write_components, time_state, free_at_time_components) = data;

        //TODO-API: Find a better way to do this.. deferred delete is fine
        let mut entities_to_free = vec![];
        for (entity, free_at_time) in free_at_time_components.iter(&entity_set) {
            if free_at_time.should_free(&time_state) {
                entities_to_free.push(entity);
            }
        }

        for e in entities_to_free {
            entity_set.enqueue_free(&e, &mut *write_components);
        }
    }
}
