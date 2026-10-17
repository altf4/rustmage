//! An in-process, deterministic rules-engine foundation. See README for the exact coverage.
// Capacity failures use typed internal unwinds to restore action transactions.
// An aborting panic profile cannot uphold the recoverable-error contract.
#[cfg(panic = "abort")]
compile_error!("rustmage requires panic=unwind for atomic recoverable capacity errors");
mod capacity;
mod shared_vec;
pub use capacity::CapacityExceeded;
mod cards;
mod engine;
mod ffi;
pub mod policy;
pub mod protocol;
mod rng;

pub use cards::{
    CATALOGUE_VERSION, CREATURE_TYPES, Card, CardInfo, CounterKind, Counters, EffectChoiceKind,
    EmblemInfo, GiftKind, Keywords, LiveStatCondition, LiveStatRecipients, LiveStatValue,
    LiveStaticEffect, Mana, ManaCost, MilledDestination, MilledFilter, RecoveryFilter,
    SearchDestination, SearchFilter, TokenInfo, is_creature_type,
};
pub use engine::*;
