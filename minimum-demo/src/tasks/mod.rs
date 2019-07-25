use minimum::systems::{async_dispatch::Task, DataRequirement, Read, Write};

use crate::resources::{InputManager, MouseButtons, PhysicsManager, RenderState, TimeState};

use crate::components;
use minimum::component::{ReadComponent, WriteComponent};
use minimum::{ComponentStorage, EntitySet};

mod imgui_tasks;
pub use imgui_tasks::ImguiBeginFrame;
pub use imgui_tasks::RenderImguiMainMenu;

mod debug_draw_tasks;
pub use debug_draw_tasks::UpdateDebugDraw;

mod input_manager_tasks;
pub use input_manager_tasks::GatherInput;

pub fn render(world: &minimum::systems::World) {
    let window = world.fetch::<winit::window::Window>();
    let mut renderer = world.fetch_mut::<crate::renderer::Renderer>();
    renderer.render(&window, &world);
}

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
        Write<minimum::EntitySet>,
        Read<InputManager>,
        Read<TimeState>,
        Read<RenderState>,
        Write<PhysicsManager>,
        ReadComponent<components::PlayerComponent>,
        WriteComponent<components::PositionComponent>,
        WriteComponent<components::VelocityComponent>,
        WriteComponent<components::DebugDrawCircleComponent>,
        WriteComponent<components::BulletComponent>,
        WriteComponent<components::FreeAtTimeComponent>,
        WriteComponent<components::PhysicsBodyComponent>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            mut entity_set,
            input_manager,
            time_state,
            render_state,
            mut physics_manager,
            player_components,
            mut position_components,
            mut velocity_components,
            mut debug_draw_circle_components,
            mut bullet_components,
            mut free_at_time_components,
            mut physics_body_components,
        ) = data;

        let dt = time_state.previous_frame_dt;

        use winit::event::VirtualKeyCode;

        let mut pending_bullets = vec![];

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

                    pending_bullets.push((pos.position(), velocity));
                }
            }
        }

        //TODO-API: Defer this to frame sync point.. we can reduce the required resources once that's done
        for pending_bullet in pending_bullets {
            crate::constructors::create_bullet(
                pending_bullet.0,
                pending_bullet.1,
                &time_state,
                &mut physics_manager,
                &mut entity_set,
                &mut *position_components,
                &mut *velocity_components,
                &mut *debug_draw_circle_components,
                &mut *bullet_components,
                &mut *free_at_time_components,
                &mut *physics_body_components,
            );
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

pub struct UpdatePhysics;
impl Task for UpdatePhysics {
    type RequiredResources = (Read<TimeState>, Write<PhysicsManager>);

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (time_state, mut physics) = data;
        physics.update(&time_state);
    }
}

pub struct UpdatePositionFromPhysics;
impl Task for UpdatePositionFromPhysics {
    type RequiredResources = (
        Read<EntitySet>,
        Read<PhysicsManager>,
        ReadComponent<components::PhysicsBodyComponent>,
        WriteComponent<components::PositionComponent>,
        WriteComponent<components::VelocityComponent>,
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            entity_set,
            physics_manager,
            physics_body_components,
            mut pos_components,
            mut vel_components,
        ) = data;

        for (entity, body_component) in physics_body_components.iter(&entity_set) {
            let body: &nphysics2d::object::RigidBody<f32> = physics_manager
                .world()
                .rigid_body(body_component.body_handle())
                .unwrap();

            if let Some(pos_component) = pos_components.get_mut(&entity) {
                *pos_component.position_mut() = body.position().translation.vector;
            }

            if let Some(vel_component) = vel_components.get_mut(&entity) {
                *vel_component.velocity_mut() = body.velocity().linear;
            }
        }
    }
}
