//! Thread-local typed providers used only by replacement integration tests.
//! The fixture preserves normal card identity and characteristics and supplies
//! static rule groups only while a single test is executing on its own thread.
use super::*;
use std::cell::Cell;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
thread_local! { static OUTSIDE_ACTIVE: Cell<bool> = const { Cell::new(false) }; }
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
thread_local! { static OUTSIDE_SELF_ACTIVE: Cell<u8> = const { Cell::new(0) }; }
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) struct OutsideSelfGuard;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Drop for OutsideSelfGuard {
    fn drop(&mut self) {
        OUTSIDE_SELF_ACTIVE.with(|v| v.set(0));
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) fn activate_outside_self() -> OutsideSelfGuard {
    OUTSIDE_SELF_ACTIVE.with(|v| assert_eq!(v.replace(1), 0));
    OutsideSelfGuard
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) fn activate_outside_self_chain() -> OutsideSelfGuard {
    OUTSIDE_SELF_ACTIVE.with(|v| assert_eq!(v.replace(2), 0));
    OutsideSelfGuard
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const OUTSIDE_SELF_CHAIN: &[SelfOutsideEntryTemplate] = &[
    SelfOutsideEntryTemplate {
        from: OutsideEntryEffectDestination::Hand,
        to: OutsideEntryEffectDestination::Hand,
    },
    SelfOutsideEntryTemplate {
        from: OutsideEntryEffectDestination::Exile,
        to: OutsideEntryEffectDestination::Hand,
    },
];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const OUTSIDE_SELF_EFFECTS: &[SelfOutsideEntryTemplate] = &[
    SelfOutsideEntryTemplate {
        from: OutsideEntryEffectDestination::Hand,
        to: OutsideEntryEffectDestination::Exile,
    },
    SelfOutsideEntryTemplate {
        from: OutsideEntryEffectDestination::Hand,
        to: OutsideEntryEffectDestination::Prevent,
    },
];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn outside_self_spell(card: Card) -> Option<Effect> {
    if Some(card) != Card::from_name("Extrapolate the Impossible") {
        return None;
    }
    match OUTSIDE_SELF_ACTIVE.with(Cell::get) {
        1 => Some(Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::RevealTwoOutsideCardsWithSelf(
                OUTSIDE_SELF_EFFECTS,
            )],
        )),
        2 => Some(Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::RevealTwoOutsideCardsWithSelf(
                OUTSIDE_SELF_CHAIN,
            )],
        )),
        _ => None,
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) struct OutsideGuard;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Drop for OutsideGuard {
    fn drop(&mut self) {
        OUTSIDE_ACTIVE.with(|v| v.set(false));
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) fn activate_outside() -> OutsideGuard {
    OUTSIDE_ACTIVE.with(|v| assert!(!v.replace(true)));
    OutsideGuard
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) struct Guard;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Drop for Guard {
    fn drop(&mut self) {
        ACTIVE.with(|v| v.set(false));
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(crate) fn activate() -> Guard {
    ACTIVE.with(|v| {
        assert!(!v.replace(true));
    });
    Guard
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const BLOCK: StaticAbilityFragment = StaticAbilityFragment::PlayerCounterProhibition {
    kind: PlayerCounterKind::Poison,
    relation: CastRelation::You,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const DOUBLE: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Poison,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: false,
    factor: 2,
    add: 0,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const PLUS: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Poison,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: false,
    factor: 1,
    add: 1,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const EFFECT_ONLY: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Poison,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: true,
    factor: 2,
    add: 0,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ENERGY_DOUBLE: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Energy,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: false,
    factor: 2,
    add: 0,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ENERGY_PLUS: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Energy,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: false,
    factor: 1,
    add: 1,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ENERGY_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::Replacement(0, ENERGY_DOUBLE),
        StaticAbilityFragment::Replacement(1, ENERGY_PLUS),
    ],
}];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const EVENT_DOUBLE: ReplacementSpec = ReplacementSpec::Proliferate {
    relation: CastRelation::You,
    factor: 2,
    add: 0,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const BLOCK_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[BLOCK],
}];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const DOUBLE_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
}];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const PLUS_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, PLUS)],
}];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const EFFECT_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, EFFECT_ONLY)],
}];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const EVENT_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, EVENT_DOUBLE)],
}];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const OUTSIDE_PREVENT: ReplacementSpec = ReplacementSpec::OutsideEntry {
    relation: CastRelation::You,
    from: OutsideEntryEffectDestination::Hand,
    to: OutsideEntryEffectDestination::Prevent,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const OUTSIDE_EXILE: ReplacementSpec = ReplacementSpec::OutsideEntry {
    relation: CastRelation::You,
    from: OutsideEntryEffectDestination::Hand,
    to: OutsideEntryEffectDestination::Exile,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const OUTSIDE_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::Replacement(0, OUTSIDE_PREVENT),
        StaticAbilityFragment::Replacement(1, OUTSIDE_EXILE),
    ],
}];
// Two distinct replacements at each destination force two public decisions.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const OUTSIDE_RETURN: ReplacementSpec = ReplacementSpec::OutsideEntry {
    relation: CastRelation::You,
    from: OutsideEntryEffectDestination::Exile,
    to: OutsideEntryEffectDestination::Hand,
};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const OUTSIDE_EXILE_PAIR: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::Replacement(0, OUTSIDE_EXILE),
        StaticAbilityFragment::Replacement(1, OUTSIDE_EXILE),
    ],
}];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const OUTSIDE_RETURN_PAIR: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::Replacement(0, OUTSIDE_RETURN),
        StaticAbilityFragment::Replacement(1, OUTSIDE_RETURN),
    ],
}];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn provider(
    card: Card,
) -> Option<(&'static [StaticAbilityGroup], &'static [ReplacementSpec])> {
    if OUTSIDE_ACTIVE.with(Cell::get) {
        if Some(card) == Card::from_name("Healer's Hawk") {
            return Some((OUTSIDE_GROUP, &[OUTSIDE_PREVENT, OUTSIDE_EXILE]));
        }
        if Some(card) == Card::from_name("Aegis Turtle") {
            return Some((OUTSIDE_EXILE_PAIR, &[OUTSIDE_EXILE, OUTSIDE_EXILE]));
        }
        if Some(card) == Card::from_name("Hill Giant") {
            return Some((OUTSIDE_RETURN_PAIR, &[OUTSIDE_RETURN, OUTSIDE_RETURN]));
        }
    }
    if !ACTIVE.with(Cell::get) {
        return None;
    }
    if Some(card) == Card::from_name("Bear Cub") {
        Some((BLOCK_GROUP, &[]))
    } else if Some(card) == Card::from_name("Hill Giant") {
        Some((DOUBLE_GROUP, &[DOUBLE]))
    } else if Some(card) == Card::from_name("Grizzly Bears") {
        Some((PLUS_GROUP, &[PLUS]))
    } else if Some(card) == Card::from_name("Aegis Turtle") {
        Some((EFFECT_GROUP, &[EFFECT_ONLY]))
    } else if Some(card) == Card::from_name("Goblin Piker") {
        Some((ENERGY_GROUP, &[ENERGY_DOUBLE, ENERGY_PLUS]))
    } else if Some(card) == Card::from_name("Serra Angel") {
        Some((EVENT_GROUP, &[EVENT_DOUBLE]))
    } else {
        None
    }
}
