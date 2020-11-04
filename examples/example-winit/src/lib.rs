// There is "dead" example code in this crate
#![allow(dead_code)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

use legion::*;

use std::collections::HashMap;

use atelier_assets::core::asset_uuid;

use minimum::components::*;

mod systems;
use systems::*;

pub mod app;

use atelier_assets::core as atelier_core;

use minimum::resources::{
    AssetResource, CameraResource, ViewportResource, DebugDraw2DResource, TimeResource,
    ComponentRegistryResource, DebugDraw3DResource,
};
use minimum::editor::EditorInspectRegistry;
use minimum::editor::EditorInspectRegistryBuilder;
use minimum::editor::EditorSelectRegistry;
use minimum::editor::resources::EditorMode;
use minimum::editor::resources::EditorStateResource;
use minimum::editor::resources::EditorDraw3DResource;
use minimum::editor::resources::EditorSelectionResource;
use skulpin::Window;
use minimum::ComponentRegistry;
use minimum::resources::editor::EditorInspectRegistryResource;
use minimum::editor::EditorSelectRegistryBuilder;

use minimum_nphysics2d::resources::PhysicsResource;
use minimum_nphysics2d::components::*;
use example_shared::resources::FpsTextResource;
use minimum_skulpin::components::*;
use atelier_assets::loader::storage::DefaultIndirectionResolver;
use atelier_assets::loader::RpcIO;
use atelier_assets::loader::Loader;
use atelier_assets::loader::storage::IndirectionResolver;

pub const GROUND_HALF_EXTENTS_WIDTH: f32 = 3.0;
pub const GRAVITY: f32 = -9.81;

/// Create the asset manager that has all the required types registered
pub fn create_asset_manager(loader: Loader, resolver: Box<dyn IndirectionResolver + Send + Sync + 'static>) -> AssetResource {
    let mut asset_manager = AssetResource::new(loader, resolver);
    asset_manager.add_storage::<minimum::pipeline::PrefabAsset>();
    asset_manager
}

pub fn create_component_registry() -> ComponentRegistry {
    minimum::ComponentRegistryBuilder::new()
        .auto_register_components()
        .add_spawn_mapping_into::<DrawSkiaCircleComponentDef, DrawSkiaCircleComponent>()
        .add_spawn_mapping_into::<DrawSkiaBoxComponentDef, DrawSkiaBoxComponent>()
        .add_spawn_mapping::<RigidBodyBallComponentDef, RigidBodyComponent>()
        .add_spawn_mapping::<RigidBodyBoxComponentDef, RigidBodyComponent>()
        .add_spawn_mapping_into::<TransformComponentDef, TransformComponent>()
        .build()
}

pub fn create_editor_selection_registry() -> EditorSelectRegistry {
    EditorSelectRegistryBuilder::new()
        .register::<DrawSkiaBoxComponent>()
        .register::<DrawSkiaCircleComponent>()
        .register_transformed::<RigidBodyBoxComponentDef, RigidBodyComponent>()
        .register_transformed::<RigidBodyBallComponentDef, RigidBodyComponent>()
        .build()
}

pub fn create_editor_inspector_registry() -> EditorInspectRegistry {
    EditorInspectRegistryBuilder::default()
        .register::<DrawSkiaCircleComponentDef>()
        .register::<DrawSkiaBoxComponentDef>()
        .register::<RigidBodyBallComponentDef>()
        .register::<RigidBodyBoxComponentDef>()
        .register::<TransformComponentDef>()
        .build()
}

pub struct DemoApp {
    update_schedules: HashMap<ScheduleCriteria, Schedule>,
    draw_schedules: HashMap<ScheduleCriteria, Schedule>,
}

impl DemoApp {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // The expected states for which we will generate schedules
        let expected_criteria = vec![
            ScheduleCriteria::new(false, EditorMode::Inactive),
            ScheduleCriteria::new(true, EditorMode::Active),
        ];

        // Populate a lookup for the schedules.. on each update/draw, we will check the current
        // state of the application, create an appropriate ScheduleCriteria, and use it to look
        // up the correct schedule to run
        let mut update_schedules = HashMap::default();
        let mut draw_schedules = HashMap::default();

        for criteria in &expected_criteria {
            update_schedules.insert(criteria.clone(), systems::create_update_schedule(&criteria));
            draw_schedules.insert(criteria.clone(), systems::create_draw_schedule(&criteria));
        }

        DemoApp {
            update_schedules,
            draw_schedules,
        }
    }

    // Determine the current state of the game
    fn get_current_schedule_criteria(resources: &Resources) -> ScheduleCriteria {
        ScheduleCriteria::new(
            resources
                .get::<TimeResource>()
                .unwrap()
                .is_simulation_paused(),
            resources
                .get::<EditorStateResource>()
                .unwrap()
                .editor_mode(),
        )
    }
}

impl app::AppHandler for DemoApp {
    fn init(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        window: &dyn Window,
    ) {
        let rpc_loader = RpcIO::new("127.0.0.1:9999".to_string()).unwrap();
        let loader = Loader::new(Box::new(rpc_loader));
        let resolver = Box::new(DefaultIndirectionResolver);
        let asset_manager = create_asset_manager(loader, resolver);
        let physics = PhysicsResource::new(glam::Vec2::unit_y() * GRAVITY);

        let camera_resource = CameraResource::new(
            glam::Vec2::new(0.0, 1.0),
            crate::GROUND_HALF_EXTENTS_WIDTH * 1.5,
        );

        let window_size = window.physical_size();
        let viewport_size_in_pixels =
            glam::Vec2::new(window_size.width as f32, window_size.height as f32);

        let mut viewport = ViewportResource::empty();
        example_shared::viewport::update_viewport(
            &mut viewport,
            viewport_size_in_pixels,
            camera_resource.position,
            camera_resource.x_half_extents,
        );

        resources.insert(EditorInspectRegistryResource::new(
            create_editor_inspector_registry(),
        ));
        resources.insert(EditorSelectionResource::new(
            create_editor_selection_registry(),
        ));
        resources.insert(ComponentRegistryResource::new(create_component_registry()));
        resources.insert(physics);
        resources.insert(FpsTextResource::new());
        resources.insert(asset_manager);
        resources.insert(EditorStateResource::new());
        resources.insert(camera_resource);
        resources.insert(viewport);
        resources.insert(DebugDraw2DResource::new());
        resources.insert(DebugDraw3DResource::new());
        resources.insert(EditorDraw3DResource::new());

        use minimum_winit::input::WinitKeyboardKey;
        use skulpin::winit::event::VirtualKeyCode;
        let keybinds = minimum::resources::editor::Keybinds {
            selection_add: WinitKeyboardKey::new(VirtualKeyCode::LShift).into(),
            selection_subtract: WinitKeyboardKey::new(VirtualKeyCode::LAlt).into(),
            selection_toggle: WinitKeyboardKey::new(VirtualKeyCode::LControl).into(),
            tool_translate: WinitKeyboardKey::new(VirtualKeyCode::Key1).into(),
            tool_scale: WinitKeyboardKey::new(VirtualKeyCode::Key2).into(),
            tool_rotate: WinitKeyboardKey::new(VirtualKeyCode::Key3).into(),
            action_quit: WinitKeyboardKey::new(VirtualKeyCode::Escape).into(),
            action_toggle_editor_pause: WinitKeyboardKey::new(VirtualKeyCode::Space).into(),
        };

        resources.insert(minimum::resources::editor::EditorSettingsResource::new(
            keybinds,
        ));

        // Start the application
        EditorStateResource::open_prefab(
            world,
            resources,
            asset_uuid!("3991506e-ed7e-4bcb-8cfd-3366b31a6439"),
        )
        .unwrap();
    }

    fn update(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
    ) {
        let current_criteria = Self::get_current_schedule_criteria(resources);
        let schedule = self.update_schedules.get_mut(&current_criteria).unwrap();
        schedule.execute(world, resources);
    }

    fn draw(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
    ) {
        let current_criteria = Self::get_current_schedule_criteria(resources);
        let schedule = self.draw_schedules.get_mut(&current_criteria).unwrap();
        schedule.execute(world, resources);
    }

    fn fatal_error(
        &mut self,
        error: &app::AppError,
    ) {
        log::error!("{}", error);
    }
}
