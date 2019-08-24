//! Components can be added or removed from entities.
//!
//! When entities are destroyed, components owned by those entities are released. Allocation and
//! deallocation by default are deferred by default. This improves efficiency, and makes
//! allocation/deallocation happen at a consistent point in the update loop,
//! which can lead to eliminating some timing bugs.
//!
//! Minimum is responsible for
//! * An interface for storing/retrieving components (and a few commonly desired implementations)
//! * A pipeline for deferred creation components (ComponentFactory)
//! * A hook for handling destruction of components (ComponentFreeHandler)
//! * Automatic destruction when entities are destroyed
//!
//! Some ECS libraries abstract/hide some of the concepts here. Minimum makes no attempt to hide the
//! underlying data structures. This is purposeful, allowing users to choose exactly what order they
//! want to grab the data in.
//!
//! A 3-way join might look something like this. In this example, we search by position first, then
//! we try to find velocity and speed multiplier components. (If the speed multiplier component doesn't
//! exist, the entity will be skipped)
//! ```
//! for (entity_index, pos) in position_components.iter_mut(&game_entities) {
//!     if let (Some(vel), mul) = (
//!         velocity_components.get(&entity_index),
//!         speed_multiplier_components.get(&entity_index),
//!     ) {
//!         println!("p {:?} v {:?} m {:?}", pos, vel, mul);
//!         let multiplier = time_state.dt * mul.map(|x| x.multiplier).unwrap_or(1.0);
//!         pos.position.x += multiplier * vel.velocity.x;
//!         pos.position.y += multiplier * vel.velocity.y;
//!     }
//! }
//! ```
//!
//! Users can implement custom storages which expose different ways to query the data

mod component_factory;
mod registry;
mod slab_storage;
mod vec_storage;

use crate::slab;
use slab::RawSlab;
use slab::RawSlabKey;

use crate::entity;
use entity::EntityHandle;

pub use component_factory::ComponentCreator;
pub use component_factory::ComponentFactory;
pub use component_factory::ComponentPrototype;
pub use registry::ComponentFreeHandler;
pub use registry::ComponentRegistry;
pub use slab_storage::SlabComponentStorage;
pub use vec_storage::VecComponentStorage;

mod basic;
pub use basic::BasicComponentFactory;
pub use basic::BasicComponentPrototype;

//TODO: Make these take some sort of private index type to prevent someone from
// trying to fetch components directly (these are not checking generation.. it's assumed
// we are calling through the entity code, which does a gen check there

/// Generalizes storage for components. Typical implementation would be a parallel array to entities (i.e. VecStorage)
/// or a pool with a 1:1 lookup table to map entities to components (SlabStorage). EntityHandle includes the concept
/// of generations in it
pub trait ComponentStorage<T>: Send + Sync
where
    T: Component,
{
    /// Create a component for the given entity
    fn allocate(&mut self, entity: &EntityHandle, data: T);

    /// Free the component that is on a given entity. This function is allowed to panic if the component
    /// does not exist. Use free_if_exists if it's unknown whether the component exists or not
    fn free(&mut self, entity: &EntityHandle);

    /// Free the component, or do nothing if the component does not exist
    fn free_if_exists(&mut self, entity: &EntityHandle);

    /// Returns true if a component of type T exists on the given entity, otherwise false
    fn exists(&self, entity: &EntityHandle) -> bool;

    /// Get a ref to the component on the entity
    fn get(&self, entity: &EntityHandle) -> Option<&T>;

    /// Get a mut ref to the component on the entity
    fn get_mut(&mut self, entity: &EntityHandle) -> Option<&mut T>;
}

/// Implementation requirements of a component.
pub trait Component: Sized + Send + Sync + 'static {
    /// This type defines where/how the component is stored.
    type Storage: ComponentStorage<Self>;
}

//These are shorthand for specifying reading/writing components from a resource map
pub type ReadComponent<T> = crate::resource::Read<<T as Component>::Storage>;
pub type WriteComponent<T> = crate::resource::Write<<T as Component>::Storage>;
pub type ReadComponentOption<T> = crate::resource::ReadOption<<T as Component>::Storage>;
pub type WriteComponentOption<T> = crate::resource::WriteOption<<T as Component>::Storage>;
