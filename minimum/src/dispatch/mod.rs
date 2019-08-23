#[cfg(feature = "async_support")]
pub mod async_dispatch;

#[cfg(feature = "async_support")]
pub mod async_dispatcher;

pub mod dispatch_control;
pub mod simple_dispatch;

pub use dispatch_control::DispatchControl;
