//! An in-process, deterministic rules-engine foundation. See README for the exact coverage.
// Capacity failures use typed internal unwinds to restore action transactions.
// An aborting panic profile cannot uphold the recoverable-error contract.
#[cfg(panic = "abort")]
compile_error!("rustmage requires panic=unwind for atomic recoverable capacity errors");
#[doc(hidden)]
pub mod capacity;
#[doc(hidden)]
pub mod shared_vec;
pub use capacity::CapacityExceeded;
#[doc(hidden)]
pub mod cards;
#[doc(hidden)]
pub mod permanent_type;
pub use permanent_type::PermanentType;
#[doc(hidden)]
pub mod card_names;
pub use card_names::CardName;
#[doc(hidden)]
pub mod engine;

#[doc(hidden)]
pub mod rng;

pub use cards::{
    BasicLandType, CREATURE_TYPES, Card, CardInfo, ChoiceColor, CounterKind, Counters,
    CreatureType, EffectChoiceKind, EmblemInfo, EntryChoiceKind, EntryChoiceValue, GiftKind,
    Keywords, LiveStatCondition, LiveStatRecipients, LiveStatValue, LiveStaticEffect, Mana,
    ManaCost, ManaRetention, MilledDestination, MilledFilter, PlayerCounterKind,
    PlayerCounterSplit, RecoveryFilter, SearchDestination, SearchFilter, TokenInfo,
    is_creature_type,
};
pub use engine::*;

pub mod policy;
pub mod protocol;
