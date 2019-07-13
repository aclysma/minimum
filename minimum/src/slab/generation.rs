use super::GenerationCounterT;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct GenerationIndex(GenerationCounterT);

pub struct Generation<T> {
    generation_index: GenerationIndex,
    value: Option<T>,
}

impl<T> Generation<T> {
    pub fn new() -> Self {
        Generation {
            generation_index: GenerationIndex(0),
            value: None,
        }
    }

    pub fn get(&self, generation: GenerationIndex) -> Option<&T> {
        //println!("get self: {} param: {}", self.generation_index.0, generation.0);

        let value = self.value.as_ref()?;
        if self.generation_index == generation {
            Some(value)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, generation: GenerationIndex) -> Option<&mut T> {
        //println!("get self: {} param: {}", self.generation_index.0, generation.0);

        let value = self.value.as_mut()?;
        if self.generation_index == generation {
            Some(value)
        } else {
            None
        }
    }

    pub fn allocate(&mut self, value: T) -> GenerationIndex {
        assert!(
            self.value.is_none(),
            "Can only allocate a generation if it's not already allocated"
        );
        self.value = Some(value);

        //println!("allocate generation {}", self.generation_index.0);
        self.generation_index
    }

    pub fn free(&mut self, generation_index: GenerationIndex) {
        assert!(
            self.value.is_some(),
            "Can only free a generation if it's not already freed"
        );
        assert!(
            self.generation_index == generation_index,
            "Can not free a generation with incorrect generation_index"
        );
        self.value = None;
        self.generation_index.0 += 1;
        //println!("free generation {}", self.generation_index.0);
    }

    pub fn is_none(&self) -> bool {
        self.value.is_none()
    }

    pub fn peek(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_get() {
        // Generations starts unallocated
        let mut value = Generation::new();
        assert!(value.get(GenerationIndex(0)).is_none());

        // Once it's allocated, the first gen index will work to access it
        let generation_index0 = value.allocate(0);
        assert!(value.get(generation_index0).is_some());

        // Now that it's free, the generation won't work
        value.free(generation_index0);
        assert!(value.get(generation_index0).is_none());

        // Allocate again, the new index works and the old one doesn't
        let generation_index1 = value.allocate(0);
        assert!(value.get(generation_index0).is_none());
        assert!(value.get(generation_index1).is_some());
    }

    #[test]
    fn test_generation_get_mut() {
        // Generations starts unallocated
        let mut value = Generation::new();
        assert!(value.get_mut(GenerationIndex(0)).is_none());

        // Once it's allocated, the first gen index will work to access it
        let generation_index0 = value.allocate(0);
        assert!(value.get_mut(generation_index0).is_some());

        // Now that it's free, the generation won't work
        value.free(generation_index0);
        assert!(value.get_mut(generation_index0).is_none());

        // Allocate again, the new index works and the old one doesn't
        let generation_index1 = value.allocate(0);
        assert!(value.get_mut(generation_index0).is_none());
        assert!(value.get_mut(generation_index1).is_some());
    }

    #[test]
    #[should_panic(expected = "Can only allocate a generation if it's not already allocated")]
    fn test_double_allocate() {
        let mut value = Generation::new();
        value.allocate(0);
        value.allocate(0);
    }

    #[test]
    #[should_panic(expected = "Can only free a generation if it's not already freed")]
    fn test_double_free() {
        let mut value = Generation::new();
        let index = value.allocate(0);

        value.free(index);
        value.free(index);
    }

    #[test]
    #[should_panic(expected = "Can not free a generation with incorrect generation_index")]
    fn test_free_wrong_index() {
        let mut value = Generation::new();
        let index = value.allocate(0);

        value.free(index);
        value.allocate(0);
        value.free(index);
    }
}
