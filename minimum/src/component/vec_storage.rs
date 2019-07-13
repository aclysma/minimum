use super::Component;
use super::ComponentStorage;
use super::EntityHandle;

pub struct VecComponentStorage<T: Component> {
    components: Vec<Option<T>>,
}

impl<T: Component> ComponentStorage<T> for VecComponentStorage<T> {
    fn new() -> Self {
        VecComponentStorage::<T> {
            components: Vec::with_capacity(32), //TODO: Hardcoded value
        }
    }

    fn allocate(&mut self, entity: &EntityHandle, data: T) {
        // If the slab keys vec isn't long enough, expand it
        if self.components.len() <= entity.index() as usize {
            // Can't use resize() because T is not guaranteed to be cloneable
            self.components.reserve(entity.index() as usize + 1);
            for _index in self.components.len()..(entity.index() as usize + 1) {
                self.components.push(None);
            }
        }

        assert!(self.components[entity.index() as usize].is_none());
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
