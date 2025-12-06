//! CRDT (Conflict-free Replicated Data Types) primitives for distributed IRC state.
//!
//! This module provides building blocks for eventually-consistent distributed
//! state synchronization between linked IRC servers.
//!
//! ## Types
//!
//! - [`LamportClock`] - Logical clock for ordering events
//! - [`GSet`] - Grow-only set (add-only, never remove)
//! - [`LwwRegister`] - Last-Writer-Wins register
//! - [`ORSet`] - Observed-Remove set (supports add and remove)

mod clock;
mod gset;
mod lww;
mod orset;

pub use clock::LamportClock;
pub use gset::GSet;
pub use lww::LwwRegister;
pub use orset::ORSet;
