// u32 should be enough, even at 120fps, one allocation per frame, it would take
// more than a year to exhaust
type GenerationCounterT = u32;

// Realistically we shouldn't have 4 billion of something.. (if that's not the case.. probably
// want bespoke storage for it in any case)
type SlabIndexT = u32;

mod gen_slab;
mod generation;
mod keyed_rc_slab;
mod raw_slab;
mod rc_slab;

pub use generation::Generation;
pub use generation::GenerationIndex;

pub use raw_slab::RawSlab;
pub use raw_slab::RawSlabKey;

pub use gen_slab::GenSlab;
pub use gen_slab::GenSlabKey;

pub use rc_slab::RcSlab;
pub use rc_slab::RcSlabEntry;
pub use rc_slab::WeakSlabEntry;

pub use keyed_rc_slab::KeyedRcSlab;
