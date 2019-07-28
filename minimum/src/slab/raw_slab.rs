use super::SlabIndexT;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct RawSlabKey<T: Sized> {
    index: SlabIndexT,
    phantom_data: PhantomData<T>,
}

impl<T: Sized> RawSlabKey<T> {
    fn new(index: SlabIndexT) -> Self {
        RawSlabKey {
            index,
            phantom_data: PhantomData,
        }
    }

    pub fn index(&self) -> SlabIndexT {
        self.index
    }
}

// The pool is responsible for allocation and deletion
pub struct RawSlab<T> {
    // List of actual components
    storage: Vec<Option<T>>,

    // List of unused components, using Vec means we use LIFO
    free_list: Vec<SlabIndexT>,
}

impl<T> RawSlab<T> {
    pub fn new() -> Self {
        let initial_count: SlabIndexT = 32;
        let mut storage = Vec::with_capacity(initial_count as usize);
        let mut free_list = Vec::with_capacity(initial_count as usize);

        // reverse count so index 0 is at the top of the free list
        for index in (0..initial_count).rev() {
            storage.push(None);
            free_list.push(index);
        }

        RawSlab { storage, free_list }
    }

    pub fn allocate(&mut self, value: T) -> RawSlabKey<T> {
        let index = self.free_list.pop();

        if index.is_none() {
            let index = self.storage.len() as SlabIndexT;
            self.storage.push(Some(value));

            return RawSlabKey::new(index);
        } else {
            // Reuse a free slot
            let index = index.unwrap();
            assert!(self.storage[index as usize].is_none());
            self.storage[index as usize] = Some(value);
            return RawSlabKey::new(index);
        }
    }

    pub fn free(&mut self, slab_key: &RawSlabKey<T>) {
        assert!(
            self.storage[slab_key.index as usize].is_some(),
            "tried to free a none value"
        );
        self.storage[slab_key.index as usize] = None;
        self.free_list.push(slab_key.index);
    }

    pub fn exists(&self, slab_key: &RawSlabKey<T>) -> bool {
        self.storage[slab_key.index as usize].is_some()
    }

    pub fn get(&self, slab_key: &RawSlabKey<T>) -> Option<&T> {
        // Non-mutable return value so we can return a ref to the value in the vec

        self.storage[slab_key.index as usize].as_ref()
    }

    pub fn get_mut(&mut self, slab_key: &RawSlabKey<T>) -> Option<&mut T> {
        // Mutable reference, and we don't want the caller messing with the Option in the vec,
        // so create a new Option with a mut ref to the value in the vec
        self.storage[slab_key.index as usize].as_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (RawSlabKey<T>, &T)> {
        self.storage
            .iter()
            .enumerate()
            .filter(|(_, value)| value.is_some())
            .map(|(index, value)| (RawSlabKey::new(index as u32), value.as_ref().unwrap()))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (RawSlabKey<T>, &mut T)> {
        self.storage
            .iter_mut()
            .enumerate()
            .filter(|(_, value)| value.is_some())
            .map(|(index, value)| (RawSlabKey::new(index as u32), value.as_mut().unwrap()))
    }

    pub fn count(&self) -> usize {
        self.storage.len() - self.free_list.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStruct {
        value: u32,
    }

    impl TestStruct {
        fn new(value: u32) -> Self {
            TestStruct { value }
        }
    }

    // Check that trivial allocate/delete works
    #[test]
    fn test_allocate_deallocate_one() {
        let mut pool = RawSlab::<TestStruct>::new();
        let value = TestStruct::new(123);
        let key = pool.allocate(value);

        assert_eq!(1, pool.count());
        pool.free(&key);
        assert_eq!(0, pool.count());
    }

    #[test]
    #[should_panic(expected = "tried to free a none value")]
    fn test_double_free() {
        let mut pool = RawSlab::<TestStruct>::new();
        let value = TestStruct::new(123);
        let key = pool.allocate(value);

        assert_eq!(1, pool.count());
        pool.free(&key);
        assert_eq!(0, pool.count());
        pool.free(&key);
    }

    // Check that allocation/deallocation in order works
    #[test]
    fn test_allocate_deallocate_fifo() {
        let mut pool = RawSlab::<TestStruct>::new();
        let mut keys = vec![];

        for i in 0..1000 {
            let value = TestStruct::new(i);
            let key = pool.allocate(value);
            keys.push(key);
        }

        assert_eq!(1000, pool.count());

        for k in &keys {
            pool.free(k);
        }

        assert_eq!(0, pool.count());
    }

    #[test]
    fn test_allocate_deallocate_lifo() {
        let mut pool = RawSlab::<TestStruct>::new();
        let mut keys = vec![];

        for i in 0..1000 {
            let value = TestStruct::new(i);
            let key = pool.allocate(value);
            keys.push(key);
        }

        assert_eq!(1000, pool.count());

        for i in (0..keys.len()).rev() {
            pool.free(&keys[i]);
        }

        assert_eq!(0, pool.count());
    }

    #[test]
    fn test_get_success() {
        let mut pool = RawSlab::<TestStruct>::new();
        let mut keys = vec![];

        for i in 0..10 {
            let value = TestStruct::new(i);
            let key = pool.allocate(value);
            keys.push(key);
        }

        assert_eq!(10, pool.count());
        assert_eq!(5, pool.get(&keys[5]).unwrap().value);
    }

    #[test]
    fn test_get_fail_out_of_range() {
        let mut pool = RawSlab::<TestStruct>::new();
        let value = TestStruct::new(123);
        let key = pool.allocate(value);
        assert_eq!(1, pool.count());

        assert!(pool.get(&key).is_some());

        pool.free(&key);
        assert_eq!(0, pool.count());

        assert!(pool.get(&key).is_none());
    }

    #[test]
    fn test_get_mut_success() {
        let mut pool = RawSlab::<TestStruct>::new();
        let mut keys = vec![];

        for i in 0..10 {
            let value = TestStruct::new(i);
            let key = pool.allocate(value);
            keys.push(key);
        }

        assert_eq!(10, pool.count());
        assert_eq!(5, pool.get_mut(&keys[5]).unwrap().value);
    }

    #[test]
    fn test_get_mut_fail_out_of_range() {
        let mut pool = RawSlab::<TestStruct>::new();
        let value = TestStruct::new(123);
        let key = pool.allocate(value);
        assert_eq!(1, pool.count());

        assert!(pool.get_mut(&key).is_some());

        pool.free(&key);
        assert_eq!(0, pool.count());

        assert!(pool.get_mut(&key).is_none());
    }
}
