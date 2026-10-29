//! Thread-local typed providers used only by replacement integration tests.
//! The fixture preserves normal card identity and characteristics and supplies
//! static rule groups only while a single test is executing on its own thread.
use super::*;
use std::cell::Cell;
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
pub(crate) struct Guard;
impl Drop for Guard { fn drop(&mut self) { ACTIVE.with(|v| v.set(false)); } }
pub(crate) fn activate() -> Guard {
    ACTIVE.with(|v| { assert!(!v.replace(true)); });
    Guard
}
const BLOCK: StaticAbilityFragment = StaticAbilityFragment::PlayerCounterProhibition {
    kind: PlayerCounterKind::Poison, relation: CastRelation::You,
};
const DOUBLE: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Poison, relation: CastRelation::You,
    placed_by: Some(CastRelation::You), effects_only: false, factor: 2, add: 0,
};
const PLUS: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Poison, relation: CastRelation::You,
    placed_by: Some(CastRelation::You), effects_only: false, factor: 1, add: 1,
};
const EFFECT_ONLY: ReplacementSpec = ReplacementSpec::PlayerCounters {
    kind: PlayerCounterKind::Poison, relation: CastRelation::You,
    placed_by: Some(CastRelation::You), effects_only: true, factor: 2, add: 0,
};
const EVENT_DOUBLE: ReplacementSpec = ReplacementSpec::Proliferate {
    relation: CastRelation::You, factor: 2, add: 0,
};
const BLOCK_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0, first_layer: AbilityLayer::Rule, fragments: &[BLOCK],
}];
const DOUBLE_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0, first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
}];
const PLUS_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0, first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, PLUS)],
}];
const EFFECT_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0, first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, EFFECT_ONLY)],
}];
const EVENT_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0, first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, EVENT_DOUBLE)],
}];
pub(super) fn provider(card: Card) -> Option<(&'static [StaticAbilityGroup], &'static [ReplacementSpec])> {
    if !ACTIVE.with(Cell::get) { return None; }
    if Some(card) == Card::from_name("Bear Cub") { Some((BLOCK_GROUP, &[])) }
    else if Some(card) == Card::from_name("Hill Giant") { Some((DOUBLE_GROUP, &[DOUBLE])) }
    else if Some(card) == Card::from_name("Grizzly Bears") { Some((PLUS_GROUP, &[PLUS])) }
    else if Some(card) == Card::from_name("Aegis Turtle") { Some((EFFECT_GROUP, &[EFFECT_ONLY])) }
    else if Some(card) == Card::from_name("Serra Angel") { Some((EVENT_GROUP, &[EVENT_DOUBLE])) }
    else { None }
}
