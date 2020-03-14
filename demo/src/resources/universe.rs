use legion::prelude::*;
use std::ops::{Deref, DerefMut};

// For now just wrap the input helper that skulpin provides
pub struct UniverseResource {
    pub universe: Universe,
}

impl UniverseResource {
    pub fn new(universe: Universe) -> Self {
        UniverseResource { universe }
    }
}

impl Deref for UniverseResource {
    type Target = Universe;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.universe
    }
}

impl DerefMut for UniverseResource {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.universe
    }
}
