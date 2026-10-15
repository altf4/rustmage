//! An in-process, deterministic rules-engine foundation. See README for the exact coverage.
mod cards;
mod engine;
mod ffi;
pub mod policy;
pub mod protocol;
mod rng;

pub use cards::{
    CATALOGUE_VERSION, CREATURE_TYPES, Card, CardInfo, CounterKind, Counters, EffectChoiceKind,
    Keywords, Mana, ManaCost, RecoveryFilter, SearchDestination, SearchFilter, TokenInfo,
    is_creature_type,
};
pub use engine::*;
