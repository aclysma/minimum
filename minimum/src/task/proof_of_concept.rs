
use super::*;

//TODO: Some phases
//struct ReceiveInput;
//struct PrePhysicsGameplay;
//struct ProcessPhysics;
//struct PostPhysicsGameply;
//struct Render;
//struct PostRender;

struct PhaseA;
impl Phase for PhaseA {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_before_phase::<PhaseB>();
    }
}

struct PhaseB;
impl Phase for PhaseB {
    fn configure(config: &mut TaskConfig) {

    }
}

struct PhaseC;
impl Phase for PhaseC {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_after_phase::<PhaseB>();
    }
}








#[derive(NamedType)]
struct DoSomething1TaskImpl;
type DoSomething1Task = ResourceTask<DoSomething1TaskImpl>;

impl ResourceTaskImpl for DoSomething1TaskImpl {

    type RequiredResources = (crate::Read<u32>, crate::Read<f32>);

    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<PhaseA>();
    }

    fn run(data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (mut u, f) = data;
        println!("DoSomething1TaskImpl {} {}", *u, *f);
    }
}

#[derive(NamedType)]
struct DoSomething2TaskImpl;
type DoSomething2Task = ResourceTask<DoSomething2TaskImpl>;

impl ResourceTaskImpl for DoSomething2TaskImpl {

    type RequiredResources = (crate::Write<u32>, crate::Read<f32>);

    fn configure(config: &mut TaskConfig) {
        config.this_uses_data_from::<DoSomething1Task>();
        config.this_runs_during_phase::<PhaseC>();
    }

    fn run(data: <Self::RequiredResources as DataRequirement>::Borrow) {
        let (mut u, f) = data;
        println!("DoSomething2TaskImpl {} {}", *u, *f);
        *u = *u + 1;
    }
}


struct ReadAllTaskImpl1;
type ReadAllTask1 = ReadAllTask<ReadAllTaskImpl1>;

impl ReadAllTaskImpl for ReadAllTaskImpl1 {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<PhaseA>();
    }

    fn run(resource_map: &ResourceMap) {
        println!("ReadAllTaskImpl1 read resource map");
    }
}

struct ReadAllTaskImpl2;
type ReadAllTask2 = ReadAllTask<ReadAllTaskImpl2>;

impl ReadAllTaskImpl for ReadAllTaskImpl2 {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<PhaseA>();
    }

    fn run(resource_map: &ResourceMap) {
        println!("ReadAllTaskImpl2 read resource map");
    }
}


struct WriteAllTaskImpl1;
type WriteAllTask1 = WriteAllTask<WriteAllTaskImpl1>;

impl WriteAllTaskImpl for WriteAllTaskImpl1 {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<PhaseA>();

    }

    fn run(resource_map: &mut ResourceMap) {
        println!("WriteAllTaskImpl1 write resource map");
    }
}

struct WriteAllTaskImpl2;
type WriteAllTask2 = WriteAllTask<WriteAllTaskImpl2>;

impl WriteAllTaskImpl for WriteAllTaskImpl2 {
    fn configure(config: &mut TaskConfig) {
        config.this_runs_during_phase::<PhaseA>();

    }

    fn run(resource_map: &mut ResourceMap) {
        println!("WriteAllTaskImpl2 write resource map");
    }
}



pub fn test() {
    // All the things we want to run in a frame
    let mut builder = TaskDependencyListBuilder::new();
    builder.add_task_factory::<DoSomething1Task>();
    builder.add_task_factory::<DoSomething2Task>();
    builder.add_task_factory::<ReadAllTask1>();
    builder.add_task_factory::<ReadAllTask2>();
    builder.add_task_factory::<WriteAllTask1>();
    builder.add_task_factory::<WriteAllTask2>();
    builder.add_phase::<PhaseA>();
    builder.add_phase::<PhaseB>();
    builder.add_phase::<PhaseC>();

    // All the resources that can be used within a frame
    let mut resource_map = ResourceMap::new();
    resource_map.insert(10 as u32);
    resource_map.insert(56.3 as f32);

    // Wrap them in a TrustCell
    let resource_map = TrustCell::new(resource_map);

    // Serialize the tasks and their dependencies
    let mut dependency_list = builder.build();

    // Generate a schedule
    let mut schedule = TaskScheduleBuilderSingleThread::new(dependency_list).build();

    // Execute it
    for i in 0..10 {
        println!("==========================");
        schedule.run(&resource_map);
    }
}