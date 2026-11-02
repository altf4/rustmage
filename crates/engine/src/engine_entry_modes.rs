//! Bounded printed entry modes, shared by actual and speculative worlds.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::{StaticAbilityFragment, StaticAbilityGroup};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn active_mode<W: CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
) -> Option<u16> {
    if !matches!(
        world.definition(source.card).entry_choice(),
        Some(EntryChoiceKind::NamedModes(_))
    ) || world.reference(source.card) != source
    {
        return None;
    }
    match world.entry_value(source) {
        Some(EntryChoiceValue::NamedMode(mode)) => Some(mode),
        _ => None,
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn group_enabled<W: CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    group: &StaticAbilityGroup,
) -> bool {
    if !matches!(
        world.definition(source.card).entry_choice(),
        Some(EntryChoiceKind::NamedModes(_))
    ) {
        return true;
    }
    group.fragments.iter().all(|fragment| match *fragment {
        StaticAbilityFragment::EntryMode(mode) => active_mode(world, source) == Some(mode),
        _ => true,
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn trigger_available(source: &ObjectView, definition: Card, index: usize) -> bool {
    crate::cards::authored::rules(definition)
        .unwrap()
        .extras
        .entry_mode_triggers
        .iter()
        .find(|(slot, _)| usize::from(*slot) == index)
        .is_none_or(|(_, mode)| source.active_entry_mode == Some(*mode))
}
