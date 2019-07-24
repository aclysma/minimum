use super::GenSlab;
use super::GenSlabKey;
use super::SlabIndexT;
use std::sync::Arc;
use std::sync::Weak;

//TODO: Since we have Arc indirection, is a generation index really necessary?
pub struct RcSlabEntry<T> {
    slab_key: Arc<GenSlabKey<T>>,
}

impl<T> RcSlabEntry<T> {
    pub fn new(slab_key: GenSlabKey<T>) -> Self {
        RcSlabEntry {
            slab_key: Arc::new(slab_key),
        }
    }

    pub fn downgrade(&self) -> WeakSlabEntry<T> {
        WeakSlabEntry::new(self)
    }
}

impl<T> Clone for RcSlabEntry<T> {
    fn clone(&self) -> Self {
        RcSlabEntry::<T> {
            slab_key: Arc::clone(&self.slab_key),
        }
    }
}

impl<T> PartialEq for RcSlabEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.slab_key == other.slab_key
    }
}

impl<T> Eq for RcSlabEntry<T> {}

impl<T> std::hash::Hash for RcSlabEntry<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.slab_key.hash(state);
    }
}

impl<T> std::fmt::Debug for RcSlabEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (*self.slab_key).fmt(f)
    }
}

pub struct WeakSlabEntry<T> {
    slab_key: Weak<GenSlabKey<T>>,
}

impl<T> WeakSlabEntry<T> {
    pub fn new(slab_entry: &RcSlabEntry<T>) -> Self {
        WeakSlabEntry {
            slab_key: Arc::downgrade(&slab_entry.slab_key),
        }
    }

    pub fn upgrade(&self) -> Option<RcSlabEntry<T>> {
        Some(RcSlabEntry {
            slab_key: self.slab_key.upgrade()?,
        })
    }

    pub fn can_upgrade(&self) -> bool {
        //self.slab_key.weak_count()
        unimplemented!()
    }
}

//impl<T> std::clone::Clone for WeakSlabEntry<T> {
//    fn clone(&self) -> Self {
//        WeakSlabEntry::<T> {
//            slab_key = Weak::<T>::clone(&self.slab_key)
//        }
//    }
//}

pub struct RcSlab<T> {
    slab: GenSlab<T>,
    entries: Vec<RcSlabEntry<T>>,
}

impl<T> RcSlab<T> {
    pub fn new() -> Self {
        let initial_count: SlabIndexT = 32;
        let entries = Vec::with_capacity(initial_count as usize);

        RcSlab::<T> {
            slab: GenSlab::<T>::new(),
            entries: entries,
        }
    }

    pub fn allocate(&mut self, value: T) -> RcSlabEntry<T> {
        let key = self.slab.allocate(value);
        let entry = RcSlabEntry::new(key);
        self.entries.push(entry.clone());
        entry
    }

    pub fn get(&self, slab_entry: &RcSlabEntry<T>) -> &T {
        self.slab.get(&*slab_entry.slab_key).unwrap()
    }

    pub fn get_mut(&mut self, slab_entry: &RcSlabEntry<T>) -> &mut T {
        self.slab.get_mut(&*slab_entry.slab_key).unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.slab.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slab.iter_mut()
    }

    pub fn count(&self) -> usize {
        self.slab.count()
    }

    pub fn update(&mut self) {
        for index in (0..self.entries.len()).rev() {
            if Arc::strong_count(&self.entries[index].slab_key) == 1 {
                self.slab.free(&self.entries[index].slab_key);
                self.entries.swap_remove(index);
            }
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

    #[test]
    fn test_rc_allocate_deallocate_one() {
        let mut pool = RcSlab::<TestStruct>::new();
        let value = TestStruct::new(123);
        {
            let _entry = pool.allocate(value);
            assert_eq!(1, pool.count());
        }

        assert_eq!(1, pool.count());
        pool.update();
        assert_eq!(0, pool.count());
    }

    #[test]
    fn test_rc_get_success() {
        let mut pool = RcSlab::<TestStruct>::new();
        let mut keys = vec![];

        for i in 0..10 {
            let value = TestStruct::new(i);
            let key = pool.allocate(value);
            keys.push(key);
        }

        assert_eq!(10, pool.count());
        assert_eq!(5, pool.get(&keys[5]).value);
    }

    #[test]
    fn test_rc_get_mut_success() {
        let mut pool = RcSlab::<TestStruct>::new();
        let mut keys = vec![];

        for i in 0..10 {
            let value = TestStruct::new(i);
            let key = pool.allocate(value);
            keys.push(key);
        }

        assert_eq!(10, pool.count());
        assert_eq!(5, pool.get_mut(&keys[5]).value);
    }
}
