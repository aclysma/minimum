#[macro_use]
extern crate log;

#[macro_use]
extern crate itertools;

extern crate nalgebra as na;
extern crate nalgebra_glm as glm;

use legion::prelude::*;

use std::collections::HashMap;
use legion::storage::ComponentTypeId;
use legion_prefab::ComponentRegistration;
use prefab_format::ComponentTypeUuid;
use atelier_assets::core::asset_uuid;

use minimum::components::*;

mod components;
use components::*;

mod resources;
use resources::*;

mod systems;
use systems::*;

mod winit_input;

pub mod math_conversions;

mod pipeline;
use pipeline::*;
use std::sync::mpsc::RecvTimeoutError::Timeout;
use std::borrow::BorrowMut;
use nphysics2d::object::RigidBodyDesc;

pub mod app;

mod winit_imgui;

use legion_transaction::CopyCloneImpl;
use legion_transaction::SpawnCloneImpl;

use atelier_assets::core as atelier_core;

use minimum::resources::{
    AssetResource, CameraResource, InputResource, ViewportResource, DebugDrawResource,
    TimeResource, ComponentRegistryResource,
};
use minimum::editor::EditorInspectRegistry;
use minimum::editor::EditorInspectRegistryBuilder;
use minimum::editor::EditorSelectRegistry;
use minimum::editor::resources::EditorMode;
use minimum::editor::resources::EditorStateResource;
use minimum::editor::resources::EditorDrawResource;
use minimum::editor::resources::EditorSelectionResource;
use minimum::resources::ViewportSize;
use skulpin::Window;
use minimum::ComponentRegistry;
use minimum::resources::editor::EditorInspectRegistryResource;
use minimum::editor::EditorSelectRegistryBuilder;

pub const GROUND_HALF_EXTENTS_WIDTH: f32 = 3.0;
pub const GRAVITY: f32 = -9.81;

/// Create the asset manager that has all the required types registered
pub fn create_asset_manager() -> AssetResource {
    let mut asset_manager = AssetResource::default();
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
        .register::<PositionComponent>()
        .register::<UniformScaleComponent>()
        .register::<NonUniformScaleComponent>()
        .register::<Rotation2DComponent>()
        .register::<RigidBodyBallComponentDef>()
        .register::<RigidBodyBoxComponentDef>()
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
        window: &Window,
    ) {
        let asset_manager = create_asset_manager();
        let physics = PhysicsResource::new(glam::Vec2::unit_y() * GRAVITY);

        let window_size = window.physical_size();
        let viewport_size = ViewportSize::new(window_size.width, window_size.height);

        let mut camera = CameraResource::new(
            glam::Vec2::new(0.0, 1.0),
            crate::GROUND_HALF_EXTENTS_WIDTH * 1.5,
        );
        let viewport = ViewportResource::new(viewport_size, camera.position, camera.x_half_extents);

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
        resources.insert(camera);
        resources.insert(viewport);
        resources.insert(DebugDrawResource::new());
        resources.insert(EditorDrawResource::new());

        // Start the application
        EditorStateResource::open_prefab(
            world,
            resources,
            asset_uuid!("3991506e-ed7e-4bcb-8cfd-3366b31a6439"),
        );
    }

    fn update(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
    ) {
        let current_criteria = Self::get_current_schedule_criteria(resources);
        let mut schedule = self.update_schedules.get_mut(&current_criteria).unwrap();
        schedule.execute(world, resources);
    }

    fn draw(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
    ) {
        let current_criteria = Self::get_current_schedule_criteria(resources);
        let mut schedule = self.draw_schedules.get_mut(&current_criteria).unwrap();
        schedule.execute(world, resources);
    }

    fn fatal_error(
        &mut self,
        error: &app::AppError,
    ) {
        log::error!("{}", error);
    }
}
