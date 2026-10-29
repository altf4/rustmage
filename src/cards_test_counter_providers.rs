//! Thread-local typed providers used only by replacement integration tests.
//! The fixture preserves normal card identity and characteristics and supplies
//! static rule groups only while a single test is executing on its own thread.
use super::*;
use std::cell::Cell;
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
thread_local! { static OUTSIDE_ACTIVE: Cell<bool> = const { Cell::new(false) }; }
pub(crate) struct OutsideGuard;
impl Drop for OutsideGuard {
    fn drop(&mut self) {
        OUTSIDE_ACTIVE.with(|v| v.set(false));
    }
}
pub(crate) fn activate_outside() -> OutsideGuard {
    OUTSIDE_ACTIVE.with(|v| assert!(!v.replace(true)));
    OutsideGuard
}
pub(crate) struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        ACTIVE.with(|v| v.set(false));
    }
}
pub(crate) fn activate() -> Guard {
    ACTIVE.with(|v| {
        assert!(!v.replace(true));
    });
    Guard
}
const BLOCK: StaticAbilityFragment = StaticAbilityFragment::PlayerCounterProhibition {
    kind: PlayerCounterKind::Poison,
    relation: CastRelation::You,
};
const DOUBLE: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Poison,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: false,
    factor: 2,
    add: 0,
};
const PLUS: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Poison,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: false,
    factor: 1,
    add: 1,
};
const EFFECT_ONLY: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Poison,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: true,
    factor: 2,
    add: 0,
};
const ENERGY_DOUBLE: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Energy,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: false,
    factor: 2,
    add: 0,
};
const ENERGY_PLUS: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Energy,
    relation: CastRelation::You,
    placed_by: Some(CastRelation::You),
    effects_only: false,
    factor: 1,
    add: 1,
};
const ENERGY_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::Replacement(0, ENERGY_DOUBLE),
        StaticAbilityFragment::Replacement(1, ENERGY_PLUS),
    ],
}];
const EVENT_DOUBLE: ReplacementSpec = ReplacementSpec::Proliferate {
    relation: CastRelation::You,
    factor: 2,
    add: 0,
};
const BLOCK_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[BLOCK],
}];
const DOUBLE_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
}];
const PLUS_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, PLUS)],
}];
const EFFECT_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, EFFECT_ONLY)],
}];
const EVENT_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, EVENT_DOUBLE)],
}];
const OUTSIDE_PREVENT: ReplacementSpec = ReplacementSpec::OutsideEntry {
    relation: CastRelation::You,
    from: OutsideEntryEffectDestination::Hand,
    to: OutsideEntryEffectDestination::Prevent,
};
const OUTSIDE_EXILE: ReplacementSpec = ReplacementSpec::OutsideEntry {
    relation: CastRelation::You,
    from: OutsideEntryEffectDestination::Hand,
    to: OutsideEntryEffectDestination::Exile,
};
const OUTSIDE_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::Replacement(0, OUTSIDE_PREVENT),
        StaticAbilityFragment::Replacement(1, OUTSIDE_EXILE),
    ],
}];
// Two distinct replacements at each destination force two public decisions.
const OUTSIDE_RETURN: ReplacementSpec = ReplacementSpec::OutsideEntry {
    relation: CastRelation::You,
    from: OutsideEntryEffectDestination::Exile,
    to: OutsideEntryEffectDestination::Hand,
};
const OUTSIDE_EXILE_PAIR: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::Replacement(0, OUTSIDE_EXILE),
        StaticAbilityFragment::Replacement(1, OUTSIDE_EXILE),
    ],
}];
const OUTSIDE_RETURN_PAIR: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::Replacement(0, OUTSIDE_RETURN),
        StaticAbilityFragment::Replacement(1, OUTSIDE_RETURN),
    ],
}];
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
