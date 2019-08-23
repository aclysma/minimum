use super::Component;
use super::ComponentStorage;
use super::EntityHandle;

/// Allows iteration of all components
pub struct VecComponentIterator<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<T>)>,
{
    slab_iter: I,
    entity_set: &'a super::entity::EntitySet,
}

impl<'a, T, I> VecComponentIterator<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<T>)>,
{
    fn new(entity_set: &'a super::entity::EntitySet, slab_iter: I) -> Self {
        VecComponentIterator {
            entity_set,
            slab_iter,
        }
    }
}

impl<'a, T, I> Iterator for VecComponentIterator<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<T>)>,
{
    type Item = (EntityHandle, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.slab_iter.next().map(|(entity_index, component)| {
            (
                self.entity_set.upgrade_index_to_handle(entity_index as u32),
                component.as_ref().unwrap(),
            )
        })
    }
}

/// Allows mutable iteration of all components
pub struct VecComponentIteratorMut<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a mut Option<T>)>,
{
    slab_iter: I,
    entity_set: &'a super::entity::EntitySet,
}

impl<'a, T, I> VecComponentIteratorMut<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a mut Option<T>)>,
{
    fn new(entity_set: &'a super::entity::EntitySet, slab_iter: I) -> Self {
        VecComponentIteratorMut {
            entity_set,
            slab_iter,
        }
    }
}

impl<'a, T, I> Iterator for VecComponentIteratorMut<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a mut Option<T>)>,
{
    type Item = (EntityHandle, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.slab_iter.next().map(|(entity_index, component)| {
            (
                self.entity_set.upgrade_index_to_handle(entity_index as u32),
                component.as_mut().unwrap(),
            )
        })
    }
}

/// Implements a component storage using a simple array parallel to the entity array. This storage
/// is fast, but can use an unnecessary amount of memory
pub struct VecComponentStorage<T: Component> {
    components: Vec<Option<T>>,
}

impl<T: Component> VecComponentStorage<T> {
    //TODO: Allow overriding capacity
    /// Create storage for T
    pub fn new() -> Self {
        VecComponentStorage::<T> {
            components: Vec::with_capacity(32),
        }
    }

    /// Iterate all Ts, returning (EntityHandle, &T) pair
    pub fn iter<'a>(
        &'a self,
        entity_set: &'a super::entity::EntitySet,
    ) -> impl Iterator<Item = (EntityHandle, &'a T)> {
        VecComponentIterator::<T, _>::new(
            entity_set,
            self.components
                .iter()
                .enumerate()
                .filter(|(_entity_index, component_key)| component_key.is_some()),
        )
    }

    /// Iterate all Ts mutably, returning (EntityHandle, &T) pair
    pub fn iter_mut<'a>(
        &'a mut self,
        entity_set: &'a super::entity::EntitySet,
    ) -> impl Iterator<Item = (EntityHandle, &'a mut T)> {
        VecComponentIteratorMut::<T, _>::new(
            entity_set,
            self.components
                .iter_mut()
                .enumerate()
                .filter(|(_entity_index, component_key)| component_key.is_some()),
        )
    }

    /// Iterate just the components
    pub fn iter_values<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        self.components
            .iter()
            .filter_map(|component_key| component_key.as_ref())
    }

    /// Iterate just the components mutably
    pub fn iter_values_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T> {
        self.components
            .iter_mut()
            .filter_map(|component_key| component_key.as_mut())
    }

    /// Removes all components of type T from all entities
    pub fn free_all(&mut self) {
        for component in &mut self.components {
            *component = None;
        }
    }

    /// Returns count of allocated components
    pub fn count(&self) -> usize {
        self.components.iter().filter(|x| x.is_some()).count()
    }
}

impl<T: Component> ComponentStorage<T> for VecComponentStorage<T> {
    fn allocate(&mut self, entity: &EntityHandle, data: T) {
        // If the slab keys vec isn't long enough, expand it
        if self.components.len() <= entity.index() as usize {
            // Can't use resize() because T is not guaranteed to be cloneable
            self.components.reserve(entity.index() as usize + 1);
            for _index in self.components.len()..(entity.index() as usize + 1) {
                self.components.push(None);
            }
        }

        assert!(self.components[entity.index() as usize].is_none()); // this is tripping, probably because i'm destroying/allocating the same entity in a single frame
        self.components[entity.index() as usize] = Some(data);
    }

    fn free(&mut self, entity: &EntityHandle) {
        assert!(self.components[entity.index() as usize].is_some());
        self.components[entity.index() as usize] = None;
    }

    fn free_if_exists(&mut self, entity: &EntityHandle) {
        if entity.index() as usize >= self.components.len() {
            return;
        }

        if self.components[entity.index() as usize].is_some() {
            self.free(entity);
        }
    }

    fn exists(&self, entity: &EntityHandle) -> bool {
        if entity.index() as usize >= self.components.len() {
            return false;
        }

        self.components[entity.index() as usize].is_some()
    }

    fn get(&self, entity: &EntityHandle) -> Option<&T> {
        if entity.index() as usize >= self.components.len() {
            return None;
        }

        self.components[entity.index() as usize].as_ref()
    }

    fn get_mut(&mut self, entity: &EntityHandle) -> Option<&mut T> {
        if entity.index() as usize >= self.components.len() {
            return None;
        }

        self.components[entity.index() as usize].as_mut()
    }
}
