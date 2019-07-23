

use minimum::systems::{DataRequirement, Read, ReadOption, async_dispatch::Task, Write};

use crate::resources::{
    ImguiManager,
    WindowInterface,
    InputManager,
    GameControl,
    TimeState,
    DebugDraw,
    MouseButtons
};

use crate::components;
use minimum::component::ComponentStorage;

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
        Read<<components::PlayerComponent as minimum::component::Component>::Storage>,
        Write<<components::PositionComponent as minimum::component::Component>::Storage>,
        Write<<components::VelocityComponent as minimum::component::Component>::Storage>,
        Write<<components::DebugDrawCircleComponent as minimum::component::Component>::Storage>
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            mut entity_set,
            input_manager,
            time_state,
            player_components,
            mut position_components,
            mut velocity_components,
            mut debug_draw_circle_components
        ) = data;

        let dt = time_state.previous_frame_dt;

        use winit::event::VirtualKeyCode;

        let mut pending_bullets = vec![];

        for (entity, p) in player_components.iter(&entity_set) {
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
                    println!("{:?}", input_manager.mouse_position());
                    pending_bullets.push((pos.position(), glm::vec2(40.0, 20.0)));
                }
            }
        }

        //TODO: Defer this to frame sync point.. we can reduce the required resources once that's done
        for pending_bullet in pending_bullets {
            crate::constructors::create_bullet(
                pending_bullet.0,
                pending_bullet.1,
                &mut entity_set,
                &mut *position_components,
                &mut *velocity_components,
                &mut *debug_draw_circle_components);
        }
    }
}



pub struct UpdatePositionWithVelocity;
impl Task for UpdatePositionWithVelocity {
    type RequiredResources = (
        Read<minimum::EntitySet>,
        Read<TimeState>,
        Write<<components::PositionComponent as minimum::component::Component>::Storage>,
        Read<<components::VelocityComponent as minimum::component::Component>::Storage>
    );

    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (
            entity_set,
            time_state,
            mut position_components,
            velocity_components,
        ) = data;

        let dt = time_state.previous_frame_dt;

        for (entity, vel) in velocity_components.iter(&entity_set) {
            if let Some(mut pos) = position_components.get_mut(&entity) {
                *pos.position_mut() += vel.velocity() * dt;
            }
        }
    }
}


//pub struct UpdateDebugCameraSettings;
//impl Task for UpdateDebugCameraSettings {
//    type RequiredResources = (
//        Read<core::TimeState>,
//        Read<input::InputManager>,
//        Read<gfx::RenderState>,
//        Write<gfx::DebugCameraSettings>,
//    );
//
//    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
//        let (time_state, input_manager, render_state, mut debug_camera_settings) = data;
//        debug_camera_settings.update_debug_camera(&render_state, &input_manager, &time_state);
//    }
//}

//pub struct PrePhysics;
//impl Task for PrePhysics {
//    type RequiredResources = (
//        Read<input::InputManager>,
//        Read<core::TimeState>,
//        Write<game::GameState>,
//        Write<physics::Physics>,
//    );
//
//    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
//        let (input_manager, time_state, mut game_state, mut physics) = data;
//
//        game_state
//            .vore
//            .pre_physics_update(&input_manager, &time_state, &mut physics);
//    }
//}
//
//pub struct Physics;
//impl Task for Physics {
//    type RequiredResources = (Read<core::TimeState>, Write<physics::Physics>);
//
//    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
//        let (time_state, mut physics) = data;
//        physics.update(&time_state);
//    }
//}
//
//pub struct PostPhysics;
//impl Task for PostPhysics {
//    type RequiredResources = (
//        Write<physics::Physics>,
//        Write<game::GameState>,
//        Write<<crate::game::PickupComponent as minimum::component::Component>::Storage>
//    );
//
//    fn run(&mut self, data: <Self::RequiredResources as DataRequirement>::Borrow) {
//        let (mut physics, mut game_state, mut pickups) = data;
//
//        game_state.vore.post_physics_update(&mut physics);
//
//        for pickup in pickups.iter_values_mut() {
//            pickup.post_physics_update(&mut physics);
//        }
//    }
//}
