//! Defines a Task and supports composing them into a game loop.
//!
//! WARNING: This module is likely to change.

//TODO: Explore tasks defining their own requirements (i.e. I execute before X, I execute after X)
// I implemented this in Helium and it seemed to work well (https://github.com/HeliumProject/Engine)
// Also cooperative MT being a good idea for games is still in my mind an open question

#[cfg(feature = "async_support")]
pub mod async_dispatch;

#[cfg(feature = "async_support")]
pub mod async_dispatcher;

pub mod dispatch_control;
pub mod simple_dispatch;

pub use dispatch_control::DispatchControl;