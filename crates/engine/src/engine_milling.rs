//! Batch milling and exact-incarnation, non-targeted recovery continuations.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn mill_batch(
        &mut self,
        players: &[Player],
        amount: MillAmount,
        controller: Player,
    ) -> usize {
        // Freeze all players' top cards before moving anything. Mandatory milling
        // does as much as possible and never attempts a draw from an empty library.
        if matches!(
            amount,
            MillAmount::RecipientGraveyard
                | MillAmount::HalfLibraryRoundedUp
                | MillAmount::HalfLibraryRoundedDown
        ) {
            return self
                .mill_batch_with_counts(players, |game, player| match amount {
                    MillAmount::RecipientGraveyard => game
                        .state
                        .objects
                        .iter()
                        .filter(|object| {
                            object.zone == Zone::Graveyard
                                && object.owner == player
                                && object.is_card()
                        })
                        .count(),
                    MillAmount::HalfLibraryRoundedUp => game.library_card_count(player).div_ceil(2),
                    MillAmount::HalfLibraryRoundedDown => game.library_card_count(player) / 2,
                    _ => unreachable!(),
                })
                .iter()
                .filter(|event| event.moved)
                .count();
        }
        let maximum = match amount {
            MillAmount::RecipientGraveyard
            | MillAmount::HalfLibraryRoundedUp
            | MillAmount::HalfLibraryRoundedDown => {
                unreachable!("per-player counts handled above")
            }
            MillAmount::LifeEvent => usize::try_from(
                self.state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .item
                    .life_event
                    .expect("saved life event")
                    .amount
                    .max(0),
            )
            .unwrap_or(usize::MAX),
            MillAmount::Fixed(n) => usize::from(n),
            MillAmount::ChosenX => {
                usize::try_from(self.state.resolving.as_ref().unwrap().item.x.unwrap_or(0))
                    .unwrap_or(usize::MAX)
            }
            MillAmount::Amount(value) => usize::try_from(
                self.item_amount(value, &self.state.resolving.as_ref().unwrap().item),
            )
            .unwrap_or(usize::MAX),
            MillAmount::ControllerLife => {
                usize::try_from(self.state.players[controller.index()].life.max(0))
                    .unwrap_or(usize::MAX)
            }
        };
        self.mill_batch_up_to(players, maximum)
            .iter()
            .filter(|event| event.moved)
            .count()
    }

    /// The rad-trigger continuation uses the exact committed destinations and
    /// public characteristics of this one mill event, including replacements.
    pub(super) fn mill_radiation(&mut self, player: Player, count: u64) -> u64 {
        let maximum = usize::try_from(count).unwrap_or(usize::MAX);
        let events = self.mill_batch_up_to(&[player], maximum);
        u64::try_from(
            events
                .iter()
                .filter(|event| {
                    event.moved
                        && self.public_milled_object(event.after)
                        && self
                            .state
                            .objects
                            .get(event.after.card as usize)
                            .is_some_and(|object| {
                                object.generation == event.after.generation
                                    && !object.is_token()
                                    && !object.is_spell_copy()
                                    && self.current_kinds(event.after.card) & 2 == 0
                            })
                })
                .count(),
        )
        .capacity("radiation mill count exhausted")
    }

    fn mill_batch_up_to(&mut self, players: &[Player], maximum: usize) -> Vec<ZoneChange> {
        self.mill_batch_with_counts(players, |_, _| maximum)
    }

    fn mill_batch_with_counts(
        &mut self,
        players: &[Player],
        maximum: impl Fn(&Self, Player) -> usize,
    ) -> Vec<ZoneChange> {
        let ids: Vec<_> = players
            .iter()
            .flat_map(|&player| {
                let count = self.replaced_mill_count(player, maximum(self, player));
                self.state.players[player.index()]
                    .library
                    .iter()
                    .rev()
                    .copied()
                    .filter(|&id| self.state.objects[id as usize].is_card())
                    .take(count)
            })
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Graveyard);
        self.state.resolving.as_mut().unwrap().milled = events
            .iter()
            .filter(|event| event.moved)
            .map(|event| event.after)
            .collect();
        self.capture_reflexive_zone_action(&events, roles::ReflexiveAction::Mill);
        self.finish_zone_events(&events, &[]);
        events
    }

    fn replaced_mill_count(&self, player: Player, count: usize) -> usize {
        if count == 0 {
            return 0;
        }
        // Each live additive replacement applies once. Additions commute, and
        // the library size bounds how many cards the resulting event can move.
        self.state.static_sources.iter().fold(count, |count, &id| {
            let source = &self.state.objects[id as usize];
            if source.controller == player {
                return count;
            }
            source
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|group| group.fragments)
                .fold(count, |count, fragment| match fragment {
                    StaticAbilityFragment::OpponentMillAdditional(additional)
                        if self.active_rule_fragment(id, |candidate| candidate == fragment) =>
                    {
                        count.saturating_add(usize::from(*additional))
                    }
                    _ => count,
                })
        })
    }

    pub(super) fn refill_empty_library_from_graveyard(
        &mut self,
        item: &SpellView,
        amount: u16,
    ) -> bool {
        if self.library_card_count(item.controller) != 0 {
            return false;
        }
        if let Some(source) = item.source
            && self.on_battlefield(source.id)
            && self.object_ref(source.id) == source.object
        {
            self.move_object(source.id, Zone::Exile);
        }
        if amount == 0 {
            return false;
        }
        self.state.resolving.as_mut().unwrap().milled = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                object.owner == item.controller
                    && object.zone == Zone::Graveyard
                    && object.is_card()
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect();
        self.start_milled_selection(MilledSelectionSpec {
            filter: MilledFilter::Any,
            count: MilledCount::Range {
                minimum: amount,
                maximum: amount,
            },
            destination: MilledDestination::LibraryTop,
        })
    }

    pub(super) fn can_recover_milled(
        &self,
        player: Player,
        filter: RecoveryFilter,
        object: ObjectRef,
    ) -> bool {
        self.state
            .resolving
            .as_ref()
            .is_some_and(|resolution| resolution.milled.contains(&object))
            && self.public_milled_object(object)
            && self
                .state
                .objects
                .get(object.card as usize)
                .is_some_and(|card| card.owner == player && filter.matches(card.card))
    }

    pub(super) fn recover_milled(&mut self, object: Option<ObjectRef>) {
        // The legal-action visitor already checked membership, generation, zone,
        // ownership and quality. Declining consumes this choice, without targeting.
        if let Some(object) = object {
            let player = self.state.resolving.as_ref().unwrap().item.controller;
            self.move_milled_selection(player, &[object], MilledDestination::Hand);
            let resolution = self.state.resolving.as_mut().unwrap();
            if let Instruction::RecoverMilledOrSkip { skip, .. } =
                resolution.program[resolution.pc - 1]
            {
                resolution.pc += skip;
            }
        }
        self.run_program();
    }
}
