//! An in-process, deterministic rules-engine foundation. See README for the exact coverage.
// Capacity failures use typed internal unwinds to restore action transactions.
// An aborting panic profile cannot uphold the recoverable-error contract.
#[cfg(panic = "abort")]
compile_error!("rustmage requires panic=unwind for atomic recoverable capacity errors");
mod capacity;
mod shared_vec;
pub use capacity::CapacityExceeded;
mod cards;
mod permanent_type;
pub use permanent_type::PermanentType;
mod card_names;
pub use card_names::CardName;
mod engine;
mod ffi;
pub mod policy;
pub mod protocol;
mod rng;

pub use cards::{
    BasicLandType, CATALOGUE_VERSION, CREATURE_TYPES, Card, CardInfo, ChoiceColor, CounterKind,
    Counters, CreatureType, EffectChoiceKind, EmblemInfo, EntryChoiceKind, EntryChoiceValue,
    GiftKind, Keywords, LiveStatCondition, LiveStatRecipients, LiveStatValue, LiveStaticEffect,
    Mana, ManaCost, ManaRetention, MilledDestination, MilledFilter, PlayerCounterKind,
    PlayerCounterSplit, RecoveryFilter, SearchDestination, SearchFilter, TokenInfo,
    is_creature_type,
};
pub use engine::*;
