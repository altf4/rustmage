//! An in-process, deterministic rules-engine foundation. See README for the exact coverage.
mod cards;
mod engine;
mod ffi;
pub mod policy;
pub mod protocol;
mod rng;

pub use cards::{Card, Mana};
pub use engine::*;
