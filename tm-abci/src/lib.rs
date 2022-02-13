#![no_std]

extern crate alloc;

#[cfg(feature = "async")]
mod async_abci;
#[cfg(feature = "async")]
pub use async_abci::Application;

#[cfg(feature = "sync")]
mod sync_abci;
#[cfg(feature = "sync")]
pub use sync_abci::SyncApplication;
