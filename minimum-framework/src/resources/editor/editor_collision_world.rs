use base::EntityHandle;

const MARGIN: f32 = 0.02;

pub struct EditorCollisionWorld {
    world: ncollide::world::CollisionWorld<f32, EntityHandle>,
}

impl EditorCollisionWorld {
    pub fn new() -> Self {
        let world = ncollide::world::CollisionWorld::new(MARGIN);

        EditorCollisionWorld { world }
    }

    pub fn update(&mut self) {
        self.world.update();
    }

    pub fn world(&self) -> &ncollide::world::CollisionWorld<f32, EntityHandle> {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut ncollide::world::CollisionWorld<f32, EntityHandle> {
        &mut self.world
    }
}
