//! Stitches together all components of minimum.
use crate::resource::{Resource, ResourceMap};

use crate::component::{
    Component, ComponentCreateQueueFlushListener, ComponentFreeHandler,
    ComponentRegistry, ComponentStorage,
};

use crate::entity::{EntityFactory, EntitySet, PendingDeleteComponent};
use crate::{TaskDependencyListBuilder, TaskDependencyList, TrustCell, TaskScheduleSingleThread, TaskScheduleBuilderSingleThread, DispatchControl};
use crate::task::TaskFactory;
use crate::task::Phase;

/// A builder for setting up a `World`
pub struct WorldBuilder {
    resource_map: ResourceMap,
    tasks: TaskDependencyListBuilder,
    default_component_registry: ComponentRegistry,
}

impl WorldBuilder {
    /// Creates an empty world builder.. without resources and without any components registered.
    /// Internal-only resources/components will be set up as well.
    pub fn new() -> Self {
        WorldBuilder {
            resource_map: ResourceMap::new(),
            tasks: TaskDependencyListBuilder::new(),
            default_component_registry: ComponentRegistry::new(),
        }
    }

    /// Add a resource to the map
    pub fn with_resource<R>(mut self, r: R) -> Self
    where
        R: Resource,
    {
        self.add_resource(r);
        self
    }

    //TODO: The storage/factory types here are rendundant and a user could possibly pass a component/storage that doesn't match
    /// Add a component type
    pub fn with_component<C: Component, S: ComponentStorage<C> + 'static>(
        mut self,
        component_storage: S,
    ) -> Self {
        self.add_component::<C, S>(component_storage);
        self
    }

    /// Add a component type, but set it up with a custom ComponentFreeHandler. This is an extension point for handling
    /// custom cleanup logic for components
    pub fn with_component_and_free_handler<
        C: Component,
        S: ComponentStorage<C> + 'static,
        F: ComponentFreeHandler<C> + 'static,
    >(
        mut self,
        component_storage: S,
    ) -> Self {
        self.resource_map.insert(component_storage);
        self.default_component_registry
            .register_component_with_free_handler::<C, F>();
        self
    }

    /// Adds a component factory. Multiple factories are allowed per component type.
    pub fn with_component_factory<F: ComponentCreateQueueFlushListener>(
        mut self,
        component_factory: F,
    ) -> Self {
        self.resource_map.insert(component_factory);
        self.default_component_registry
            .register_component_factory::<F>();
        self
    }

    /// Add a task
    pub fn with_task<T>(mut self) -> Self
        where
            T: TaskFactory
    {
        self.add_task::<T>();
        self
    }

    /// Add the standard phases
    pub fn with_default_phases(mut self) -> Self {
        self.add_default_phases();
        self
    }

    /// Add a custom phase
    pub fn with_phase<P>(mut self) -> Self
        where
            P: Phase
    {
        self.add_phase::<P>();
        self
    }

    /// Adds a resource type/instance
    pub fn add_resource<R>(&mut self, r: R)
    where
        R: Resource,
    {
        self.resource_map.insert(r);
    }

    //TODO: The storage/factory types here are rendundant and a user could possibly pass a component/storage that doesn't match
    /// Add a component type
    pub fn add_component<C: Component, S: ComponentStorage<C> + 'static>(
        &mut self,
        component_storage: S,
    ) {
        self.resource_map.insert(component_storage);
        self.default_component_registry.register_component::<C>();
    }

    pub fn add_task<T>(&mut self)
    where
        T: TaskFactory
    {
        self.tasks.add_task::<T>();
    }

    pub fn add_default_phases(&mut self) {
        self.add_phase::<crate::task::PhaseFrameBegin>();
        self.add_phase::<crate::task::PhaseGatherInput>();
        self.add_phase::<crate::task::PhasePrePhysicsGameplay>();
        self.add_phase::<crate::task::PhasePhysics>();
        self.add_phase::<crate::task::PhasePostPhysicsGameplay>();
        self.add_phase::<crate::task::PhasePreRender>();
        self.add_phase::<crate::task::PhaseRender>();
        self.add_phase::<crate::task::PhasePostRender>();
        self.add_phase::<crate::task::PhaseEndFrame>();
    }

    pub fn add_phase<P>(&mut self)
    where
        P: Phase
    {
        self.tasks.add_phase::<P>();
    }

    /// Constructs a resource map with all minimum types properly set up
    pub fn build(mut self) -> World {
        self.add_resource(EntityFactory::new());
        self.add_resource(DispatchControl::new(0));
        self.add_component(<PendingDeleteComponent as Component>::Storage::new());

        // Give the component registry to an entity set and add the entity set to the resources
        let entity_set = EntitySet::new(self.default_component_registry);
        self.resource_map.insert(entity_set);

        // Build the task dependency list
        let task_list = self.tasks.build();

        //TODO: Should conversion to dependency list happen later?
        World {
            resource_map: self.resource_map,
            task_list
        }
    }

    pub fn build_update_loop_single_threaded(self) -> UpdateLoopSingleThreaded {
        let world = self.build();
        UpdateLoopSingleThreaded::new(world)
    }
}

/// This is an intermediate data structure between the world builder and an update loop. Generally,
/// you would construct it with a world builder, then create an update loop with it.
pub struct World {
    pub resource_map: ResourceMap,
    pub task_list: TaskDependencyList
}

pub struct UpdateLoopSingleThreaded {
    resource_map: TrustCell<ResourceMap>,
    schedule: TaskScheduleSingleThread
}

impl UpdateLoopSingleThreaded {
    pub fn new(world: World) -> Self {
        UpdateLoopSingleThreaded {
            resource_map: TrustCell::new(world.resource_map),
            schedule: TaskScheduleBuilderSingleThread::new(world.task_list).build()
        }
    }

    pub fn step(&self) {
        self.schedule.step(&self.resource_map);
    }

    pub fn run(&self) {
        loop {
            self.schedule.step(&self.resource_map);

            if self.resource_map.borrow().fetch::<DispatchControl>().should_end_game_loop() {
                break;
            }
        }
    }

    pub fn into_resource_map(self) -> ResourceMap {
        self.resource_map.into_inner()
    }
}
