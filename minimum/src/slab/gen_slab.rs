use super::Generation;
use super::GenerationIndex;
use super::SlabIndexT;
use std::marker::PhantomData;

//TODO: Do I need something that doesn't have generations in it?
// Maybe this should be a DenseVec instead of a slab

#[derive(Copy, Eq)]
pub struct GenSlabKey<T> {
    index: SlabIndexT,
    generation_index: GenerationIndex,
    phantom_data: PhantomData<T>,
}

impl<T> GenSlabKey<T> {
    fn new(index: SlabIndexT, generation_index: GenerationIndex) -> GenSlabKey<T> {
        GenSlabKey::<T> {
            index,
            generation_index,
            phantom_data: PhantomData,
        }
    }
}

impl<T> GenSlabKey<T> {
    pub fn index(&self) -> SlabIndexT {
        self.index
    }
}

impl<T> Clone for GenSlabKey<T> {
    fn clone(&self) -> Self {
        GenSlabKey {
            index: self.index,
            generation_index: self.generation_index,
            phantom_data: PhantomData,
        }
    }
}

impl<T> PartialEq for GenSlabKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation_index == other.generation_index
    }
}

impl<T> std::hash::Hash for GenSlabKey<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation_index.hash(state);
    }
}

impl<T> std::fmt::Debug for GenSlabKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Index: {} Generation: {:?}",
            self.index, self.generation_index
        )
    }
}

// The pool is responsible for allocation and deletion
pub struct GenSlab<T> {
    // List of actual components
    storage: Vec<Generation<T>>,

    // List of unused components, using VecDeque means we reuse values FIFO. This is helpful
    // for debugging and slows down how quickly we go through generations
    free_list: Vec<SlabIndexT>,
}

impl<T> GenSlab<T> {
    pub fn new() -> Self {
        let initial_count: SlabIndexT = 32;
        let mut storage = Vec::with_capacity(initial_count as usize);
        let mut free_list = Vec::with_capacity(initial_count as usize);

        // reverse count so index 0 is at the top of the free list
        for index in (0..initial_count).rev() {
            storage.push(Generation::<T>::new());
            free_list.push(index);
        }

        GenSlab { storage, free_list }
    }

    pub fn allocate(&mut self, value: T) -> GenSlabKey<T> {
        let index = self.free_list.pop();

        if index.is_none() {
            // Insert a new value
            let mut generation = Generation::new();
            let generation_index = generation.allocate(value);

            let index = self.storage.len() as SlabIndexT;
            self.storage.push(generation);

            //println!("new slab index {}", index);
            return GenSlabKey::new(index, generation_index);
        } else {
            // Reuse a free slot
            let index = index.unwrap();
            //println!("reuse slab index {}", index);
            assert!(self.storage[index as usize].is_none());
            let generation_index = self.storage[index as usize].allocate(value);
            return GenSlabKey::new(index, generation_index);
        }
    }

    pub fn free(&mut self, slab_key: &GenSlabKey<T>) {
        //println!("push slab index {}", slab_key.index);
        assert!(
            self.storage[slab_key.index as usize]
                .get(slab_key.generation_index)
                .is_some(),
            "tried to free a none value"
        );
        self.storage[slab_key.index as usize].free(slab_key.generation_index);
        self.free_list.push(slab_key.index);
    }

    pub fn get(&self, slab_key: &GenSlabKey<T>) -> Option<&T> {
        // Non-mutable return value so we can return a ref to the value in the vec
        self.storage[slab_key.index as usize].get(slab_key.generation_index)
    }

    pub fn get_mut(&mut self, slab_key: &GenSlabKey<T>) -> Option<&mut T> {
        // Mutable reference, and we don't want the caller messing with the Option in the vec,
        // so create a new Option with a mut ref to the value in the vec
        self.storage[slab_key.index as usize].get_mut(slab_key.generation_index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.storage.iter().filter_map(|x| x.peek())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.storage.iter_mut().filter_map(|x| x.peek_mut())
    }

    pub fn count(&self) -> usize {
        self.storage.len() - self.free_list.len()
    }

    pub fn upgrade_index_to_handle(&self, index: SlabIndexT) -> Option<GenSlabKey<T>> {
        let index_usize = index as usize;
        if !self.storage[index_usize].is_none() {
            let generation_index = self.storage[index_usize].generation_index();
            Some(GenSlabKey::new(index, generation_index))
        } else {
            None
        }
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
        let mut pool = GenSlab::<TestStruct>::new();
        let value = TestStruct::new(123);
        let key = pool.allocate(value);

        assert_eq!(1, pool.count());
        pool.free(&key);
        assert_eq!(0, pool.count());
    }

    #[test]
    #[should_panic(expected = "tried to free a none value")]
    fn test_double_free() {
        let mut pool = GenSlab::<TestStruct>::new();
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
        let mut pool = GenSlab::<TestStruct>::new();
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
        let mut pool = GenSlab::<TestStruct>::new();
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
        let mut pool = GenSlab::<TestStruct>::new();
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
        let mut pool = GenSlab::<TestStruct>::new();
        let value = TestStruct::new(123);
        let key = pool.allocate(value);
        assert_eq!(1, pool.count());

        assert!(pool.get(&key).is_some());

        pool.free(&key);
        assert_eq!(0, pool.count());

        assert!(pool.get(&key).is_none());
    }

    #[test]
    fn test_get_fail_generation() {
        let mut pool = GenSlab::<TestStruct>::new();
        let value1 = TestStruct::new(1);
        let first_key = pool.allocate(value1);
        pool.free(&first_key);

        let value2 = TestStruct::new(2);
        let second_key = pool.allocate(value2);

        assert_eq!(first_key.index, second_key.index);
        assert_ne!(first_key.generation_index, second_key.generation_index);

        assert_eq!(2, pool.get(&second_key).unwrap().value);
        assert!(pool.get(&first_key).is_none());
    }

    #[test]
    fn test_get_mut_success() {
        let mut pool = GenSlab::<TestStruct>::new();
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
        let mut pool = GenSlab::<TestStruct>::new();
        let value = TestStruct::new(123);
        let key = pool.allocate(value);
        assert_eq!(1, pool.count());

        assert!(pool.get_mut(&key).is_some());

        pool.free(&key);
        assert_eq!(0, pool.count());

        assert!(pool.get_mut(&key).is_none());
    }

    #[test]
    fn test_get_mut_fail_generation() {
        let mut pool = GenSlab::<TestStruct>::new();
        let value1 = TestStruct::new(1);
        let first_key = pool.allocate(value1);
        pool.free(&first_key);

        let value2 = TestStruct::new(2);
        let second_key = pool.allocate(value2);

        assert_eq!(first_key.index, second_key.index);
        assert_ne!(first_key.generation_index, second_key.generation_index);

        assert_eq!(2, pool.get_mut(&second_key).unwrap().value);
        assert!(pool.get_mut(&first_key).is_none());
    }
}
