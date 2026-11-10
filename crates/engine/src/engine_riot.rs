//! CR 702.136: each Riot is an independent as-entry replacement.
use super::ability_world::CharacteristicWorld;
use super::entries::{EntryBatch, EntryMove};
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Preparation {
    pub remaining: usize,
    pub can_add_counter: bool,
    pub haste: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ChoiceOrder {
    pub order: Vec<usize>,
    pub next: usize,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn grants_riot<W: CharacteristicWorld + ?Sized>(world: &W) -> bool {
    world.static_sources().iter().any(|&id| {
        let source = world.object(id);
        source.zone == Zone::Battlefield
            && source.card.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|f| {
                    matches!(
                        f,
                        StaticAbilityFragment::GrantRiotToOtherControlledSubtype(_)
                    )
                })
            })
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn potential(m: &EntryMove, grant_possible: bool) -> bool {
    !m.skipped
        && m.before.zone != Zone::Battlefield
        && m.destination == Zone::Battlefield
        && (grant_possible
            || m.before.keywords.contains(Keywords::RIOT)
            || m.before.card.keywords().contains(Keywords::RIOT)
            || m.copy_preparation
                .as_ref()
                .and_then(|copy| copy.recipe)
                .is_some_and(|recipe| {
                    recipe.intrinsic_keywords().contains(Keywords::RIOT)
                        || recipe
                            .overrides
                            .is_some_and(|values| values.keywords.contains(Keywords::RIOT))
                }))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl EntryMove {
    pub(super) fn apply_riot(&mut self, counters: usize, haste: usize) {
        let riot = self.riot.as_mut().expect("prepared Riot");
        debug_assert!(counters == 0 || riot.can_add_counter);
        riot.remaining -= counters + haste;
        riot.haste |= haste != 0;
        self.counters.add(
            CounterKind::PlusOne,
            u64::try_from(counters).capacity("Riot counter amount exhausted"),
        );
    }

    pub(super) fn has_riot_haste(&self) -> bool {
        self.riot.is_some_and(|riot| riot.haste)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl EntryBatch {
    pub(super) fn current_riot(&self) -> Option<usize> {
        let riot = self.riot.as_ref()?;
        riot.order.get(riot.next).copied()
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl roles::RoleState {
    pub(super) fn grant_riot_haste(&mut self, recipient: ObjectRef) {
        let timestamp = self.next_timestamp();
        self.keyword_effects.push(KeywordEffectView {
            recipient,
            origin: KeywordEffectOrigin::Riot,
            keywords: Keywords::HASTE,
            rules: Keywords::NONE,
            timestamp,
            expires_turn: None,
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EntryRiotChoiceView {
    pub entrant: ObjectRef,
    pub remaining: usize,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn may_enter_with_riot(&self, card: Card) -> bool {
        card.keywords().contains(Keywords::RIOT) || grants_riot(self)
    }

    pub(super) fn advance_entry_riot(&mut self) -> bool {
        while let Some(index) = self
            .state
            .entry_batch
            .as_deref()
            .and_then(EntryBatch::current_riot)
        {
            let movement = &self.state.entry_batch.as_ref().unwrap().moves[index];
            let offer = if movement.skipped || movement.destination != Zone::Battlefield {
                None
            } else {
                movement.riot.or_else(|| self.preview_entry_riot(movement))
            };
            let batch = self.state.entry_batch.as_mut().unwrap();
            let movement = &mut batch.moves[index];
            if let Some(offer) = offer.filter(|offer| offer.remaining != 0) {
                movement.riot = Some(offer);
                if offer.can_add_counter {
                    self.state.pending = Pending::EntryRiot {
                        player: movement.controller,
                    };
                    return true;
                }
                // CR702.136b forces haste when no counter can be placed.
                movement.apply_riot(0, offer.remaining);
            }
            batch.riot.as_mut().unwrap().next += 1;
        }
        false
    }

    pub(super) fn choose_entry_riot(&mut self, counter: bool) {
        let batch = self.state.entry_batch.as_mut().unwrap();
        let index = batch.current_riot().unwrap();
        batch.moves[index].apply_riot(usize::from(counter), usize::from(!counter));
    }

    pub(super) fn entry_riot_view(&self, viewer: Player) -> Option<EntryRiotChoiceView> {
        if !matches!(self.state.pending, Pending::EntryRiot { .. }) {
            return None;
        }
        let batch = self.state.entry_batch.as_ref()?;
        let movement = &batch.moves[batch.current_riot()?];
        entries::visible_mask(
            movement,
            self.information_mask(viewer),
            &self.state.known_hand,
        )
        .then(|| EntryRiotChoiceView {
            entrant: movement.before.reference,
            remaining: movement.riot.unwrap().remaining,
        })
    }
}
