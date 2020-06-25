use glam::Vec2;
use nphysics2d::object::{DefaultBodySet, DefaultColliderSet, DefaultBodyHandle};
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};

use crossbeam_channel::{Sender, Receiver};

// Handles setting up the physics system and stepping it
pub struct PhysicsResource {
    pub geometrical_world: DefaultGeometricalWorld<f32>,
    pub mechanical_world: DefaultMechanicalWorld<f32>,
    pub bodies: DefaultBodySet<f32>,
    pub colliders: DefaultColliderSet<f32>,
    pub joint_constraints: DefaultJointConstraintSet<f32>,
    pub force_generators: DefaultForceGeneratorSet<f32>,
    pub delete_body_tx: Sender<DefaultBodyHandle>,
    pub delete_body_rx: Receiver<DefaultBodyHandle>,
}

impl PhysicsResource {
    pub fn new(gravity: Vec2) -> Self {
        let geometrical_world = DefaultGeometricalWorld::<f32>::new();
        let mechanical_world =
            DefaultMechanicalWorld::new(crate::math_conversions::vec2_glam_to_glm(gravity));

        let bodies = DefaultBodySet::<f32>::new();
        let colliders = DefaultColliderSet::new();
        let joint_constraints = DefaultJointConstraintSet::<f32>::new();
        let force_generators = DefaultForceGeneratorSet::<f32>::new();

        let (delete_body_tx, delete_body_rx) = crossbeam_channel::unbounded();

        PhysicsResource {
            geometrical_world,
            mechanical_world,
            bodies,
            colliders,
            joint_constraints,
            force_generators,
            delete_body_tx,
            delete_body_rx,
        }
    }

    pub fn delete_body_tx(&self) -> &Sender<DefaultBodyHandle> {
        &self.delete_body_tx
    }

    fn handle_deletes(&mut self) {
        // Delete any bodies that were destroyed since the previous update
        for body_to_delete in self.delete_body_rx.try_iter() {
            self.bodies.remove(body_to_delete);

            // This is a workaround for this issue: https://github.com/rustsim/nphysics/issues/248
            // It's not a long term fix since a linear search across all colliders to find the ones
            // attached to this body is expensive. This is only necessary if creating/destroying
            // entities in the same frame (between step() and maintain() calls)
            let mut colliders_to_remove = vec![];
            for (collider_handle, collider) in self.colliders.iter() {
                if collider.body() == body_to_delete {
                    colliders_to_remove.push(collider_handle);
                }
            }

            for collider_to_remove in colliders_to_remove {
                self.colliders.remove(collider_to_remove);
            }
        }
    }

    pub fn maintain(&mut self) {
        self.handle_deletes();

        self.mechanical_world.maintain(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
        );
    }

    pub fn step(&mut self) {
        self.handle_deletes();

        // Run the simulation.
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        );
    }
}
