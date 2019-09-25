//! A collection of utilities used within minimum

#[cfg(feature = "std")]
mod scope_timer;
#[cfg(feature = "std")]
pub use scope_timer::ScopeTimer;

mod trust_cell;
pub use trust_cell::Ref as TrustCellRef;
pub use trust_cell::RefMut as TrustCellRefMut;
pub use trust_cell::TrustCell;
