use super::Component;
use super::ComponentStorage;
use super::EntityHandle;
use super::RawSlab;
use super::RawSlabKey;

/// Allows iteration of all components
pub struct SlabComponentIterator<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<RawSlabKey<T>>)>,
{
    slab_iter: I,
    entity_set: &'a super::entity::EntitySet,
    raw_slab: &'a RawSlab<T>,
}

impl<'a, T, I> SlabComponentIterator<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<RawSlabKey<T>>)>,
{
    fn new(
        raw_slab: &'a RawSlab<T>,
        entity_set: &'a super::entity::EntitySet,
        slab_iter: I,
    ) -> Self {
        SlabComponentIterator {
            entity_set,
            slab_iter,
            raw_slab,
        }
    }
}

impl<'a, T, I> Iterator for SlabComponentIterator<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<RawSlabKey<T>>)>,
{
    type Item = (EntityHandle, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.slab_iter.next().map(|(entity_index, component_key)| {
            (
                self.entity_set.upgrade_index_to_handle(entity_index as u32),
                self.raw_slab.get(&component_key.as_ref().unwrap()).unwrap(),
            )
        })
    }
}

/// Allows mutable iteration of all components
pub struct SlabComponentIteratorMut<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<RawSlabKey<T>>)>,
{
    slab_iter: I,
    entity_set: &'a super::entity::EntitySet,
    raw_slab: &'a mut RawSlab<T>,
}

impl<'a, T, I> SlabComponentIteratorMut<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<RawSlabKey<T>>)>,
{
    fn new(
        raw_slab: &'a mut RawSlab<T>,
        entity_set: &'a super::entity::EntitySet,
        slab_iter: I,
    ) -> Self {
        SlabComponentIteratorMut {
            entity_set,
            slab_iter,
            raw_slab,
        }
    }
}

impl<'a, T, I> Iterator for SlabComponentIteratorMut<'a, T, I>
where
    T: Component,
    I: Iterator<Item = (usize, &'a Option<RawSlabKey<T>>)>,
{
    type Item = (EntityHandle, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        //unimplemented!();
        self.slab_iter.next().map(|(entity_index, component_key)| {
            let component_key = &component_key.as_ref().unwrap();

            // This requires unsafe code because we need to return a mut ref to an element  in
            // raw_slab, and yet we need to retain a mut ref to raw_slab so that when next() is
            // called again, we can return the next value as a ref.
            let component: &mut T = self.raw_slab.get_mut(component_key).unwrap();
            let component: &'a mut T = unsafe { std::mem::transmute(component) };

            (
                self.entity_set.upgrade_index_to_handle(entity_index as u32),
                component,
            )
        })
    }
}

/// Implements dense storage of components. slab holds the component values, and slab_keys is a
/// parallel array with entities. This allows 1:1 lookup, but with reduced memory requirements
pub struct SlabComponentStorage<T: Component> {
    slab: RawSlab<T>,
    slab_keys: Vec<Option<RawSlabKey<T>>>,
}

impl<T: Component> SlabComponentStorage<T> {
    //TODO: Allow overriding capacity
    /// Create storage for T
    pub fn new() -> Self {
        SlabComponentStorage::<T> {
            slab: RawSlab::new(),
            slab_keys: Vec::with_capacity(32),
        }
    }

    /// Iterate all Ts, returning (EntityHandle, &T) pair
    pub fn iter<'a>(
        &'a self,
        entity_set: &'a super::entity::EntitySet,
    ) -> impl Iterator<Item = (EntityHandle, &'a T)> {
        SlabComponentIterator::<T, _>::new(
            &self.slab,
            entity_set,
            self.slab_keys
                .iter()
                .enumerate()
                .filter(|(_entity_index, component_key)| component_key.is_some()),
        )
    }

    /// Iterate all Ts mutably, returning (EntityHandle, &mut T) pair
    pub fn iter_mut<'a>(
        &'a mut self,
        entity_set: &'a super::entity::EntitySet,
    ) -> impl Iterator<Item = (EntityHandle, &'a mut T)> {
        SlabComponentIteratorMut::<T, _>::new(
            &mut self.slab,
            entity_set,
            self.slab_keys
                .iter()
                .enumerate()
                .filter(|(_entity_index, component_key)| component_key.is_some()),
        )
    }

    /// Iterate just the components
    pub fn iter_values(& self) -> impl Iterator<Item = &T> {
        self.slab.iter().map(|(_key, value)| value)
    }

    /// Iterate just the components mutably
    pub fn iter_values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slab.iter_mut().map(|(_key, value)| value)
    }

    /// Removes all components of type T from all entities
    pub fn free_all(&mut self) {
        //TODO: This is not calling the free handler
        for slab_key in &mut self.slab_keys {
            if let Some(key) = &*slab_key {
                self.slab.free(&key);
            }
            *slab_key = None;
        }
    }

    /// Returns count of allocated components
    pub fn count(&self) -> usize {
        self.slab.count()
    }
}

impl<T: Component> ComponentStorage<T> for SlabComponentStorage<T> {
    fn allocate(&mut self, entity: &EntityHandle, data: T) {
        let slab_key = self.slab.allocate(data);

        // If the slab keys vec isn't long enough, expand it
        if self.slab_keys.len() <= entity.index() as usize {
            // Can't use resize() because T is not guaranteed to be cloneable
            self.slab_keys.reserve(entity.index() as usize + 1);
            for _index in self.slab_keys.len()..(entity.index() as usize + 1) {
                self.slab_keys.push(None);
            }
        }

        assert!(self.slab_keys[entity.index() as usize].is_none());
        self.slab_keys[entity.index() as usize] = Some(slab_key);
    }

    fn free(&mut self, entity: &EntityHandle) {
        //TODO: This assumes the caller already ran the free handler, would like to rework this API
        // since it's a bit dangerous
        assert!(self.slab_keys[entity.index() as usize].is_some());
        self.slab
            .free(self.slab_keys[entity.index() as usize].as_ref().unwrap());
        self.slab_keys[entity.index() as usize] = None;
    }

    fn free_if_exists(&mut self, entity: &EntityHandle) {
        if entity.index() as usize >= self.slab_keys.len() {
            return;
        }

        if self.slab_keys[entity.index() as usize].is_some() {
            self.free(entity);
        }
    }

    fn exists(&self, entity: &EntityHandle) -> bool {
        if entity.index() as usize >= self.slab_keys.len() {
            return false;
        }

        self.slab_keys[entity.index() as usize].is_some()
    }

    fn get(&self, entity: &EntityHandle) -> Option<&T> {
        if entity.index() as usize >= self.slab_keys.len() {
            return None;
        }

        self.slab
            .get(self.slab_keys[entity.index() as usize].as_ref()?)
    }

    fn get_mut(&mut self, entity: &EntityHandle) -> Option<&mut T> {
        if entity.index() as usize >= self.slab_keys.len() {
            return None;
        }

        self.slab
            .get_mut(self.slab_keys[entity.index() as usize].as_ref()?)
    }
}
