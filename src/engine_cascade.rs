//! Cascade uses a frozen cast event and exact public exiled incarnations.
use super::*;
use crate::cards::{CastFilter, StaticAbilityFragment};
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CascadeView {
    pub controller: Player,
    pub maximum_mana_value: u64,
    pub exiled: Vec<ObjectRef>,
}

impl Game {
    /// A granted spell ability belongs to the spell, with immutable provenance
    /// from the granting permanent. Later loss of that permanent cannot erase it.
    pub(super) fn queue_granted_spell_trigger(
        &mut self,
        spell: &SpellView,
        grant: AttachmentTriggerView,
        ability: u16,
    ) {
        let mut trigger = spell.clone();
        trigger.id = self.next_stack_id();
        trigger.kind = StackKind::TriggeredAbility { ability };
        trigger.targets.clear();
        trigger.mode = None;
        trigger.cast_event = spell.actual_cast_event().map(Box::new);
        trigger.cast = None;
        trigger.is_copy = false;
        trigger.related_stack = None;
        trigger.source_ability_origin = grant.origin;
        trigger.attachment_trigger = Some(Box::new(grant));
        self.state.pending_triggers.push(trigger);
    }

    pub(super) fn queue_hand_cascade(&mut self, index: usize) {
        let spell = &self.state.stack[index];
        if spell.is_copy
            || !spell.cast.is_some_and(|c| c.origin == Zone::Hand)
            || !spell.source.is_some_and(|source| CastFilter::InstantSorcery.matches(source.card))
        {
            return;
        }
        let spell = spell.clone();
        let mut grants = Vec::new();
        for &id in &self.state.static_sources {
            let source = self.object_view(id);
            if source.zone != Zone::Battlefield
                || source.controller != spell.controller
                || !source.printed_abilities_available()
            {
                continue;
            }
            for group in source.card.static_ability_groups() {
                for fragment in group.fragments {
                    if let StaticAbilityFragment::CascadeFromHand { ability } = *fragment {
                        grants.push((
                            AttachmentTriggerView {
                                grantor: source.reference,
                                granting_card: source.card,
                                recipient: spell.source.expect("cast spell has source").object,
                                defender: None,
                                granted: true,
                                origin: source.copied_grant_origin(),
                            },
                            ability,
                        ));
                    }
                }
            }
        }
        for (grant, ability) in grants {
            self.queue_granted_spell_trigger(&spell, grant, ability);
        }
    }

    pub(super) fn start_cascade(&mut self, item: &SpellView) -> bool {
        let maximum = item
            .cast_event
            .as_ref()
            .map_or_else(|| item.mana_value(), |event| event.mana_value);
        let mut exiled = Vec::new();
        let mut eligible = None;
        while let Some(card) = self.state.players[item.controller.index()]
            .library
            .last()
            .copied()
        {
            let definition = self.state.objects[card as usize].card;
            let events = self.move_objects_raw(&[card], Zone::Exile);
            let moved = events
                .iter()
                .find(|event| event.moved && event.destination == Zone::Exile)
                .map(|event| event.after);
            self.finish_zone_events(&events, &[]);
            if let Some(object) = moved {
                exiled.push(object);
                if !definition.is_land() && definition.fixed_mana_value() < maximum {
                    eligible = Some(object);
                    break;
                }
            } else if self.state.players[item.controller.index()].library.last() == Some(&card) {
                // A prevented movement cannot make progress through this library.
                break;
            }
        }
        self.state.resolving.as_mut().unwrap().cascade = Some(Box::new(CascadeView {
            controller: item.controller,
            maximum_mana_value: maximum,
            exiled,
        }));
        if let Some(object) = eligible {
            self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
                siege_transformed: false,
                paid: None,
                casts_remaining: None,
                pending_cast: None,
                alternative_cost: None,
                maximum_mana_value: Some(maximum),
                origin: Zone::Exile,
                exile_on_graveyard: false,
                allow_lands: false,
                player: item.controller,
                remaining: vec![object],
            }));
            self.state.pending = Pending::ResolutionCasts {
                player: item.controller,
            };
            return true;
        }
        self.finish_cascade();
        false
    }

    pub(super) fn finish_cascade(&mut self) {
        if self.state.resolution_free_casts.is_some() {
            return;
        }
        let Some(rows) = self.state.resolving.as_mut().and_then(|r| r.cascade.take()) else {
            return;
        };
        let mut cards: Vec<_> = rows
            .exiled
            .iter()
            .filter(|&&r| {
                self.object_ref(r.card) == r
                    && self.state.objects[r.card as usize].zone == Zone::Exile
            })
            .map(|r| r.card)
            .collect();
        if cards.is_empty() {
            return;
        }
        self.state.rng.shuffle(&mut cards);
        let player = rows.controller;
        self.forget_library_top(player);
        let events = self.move_objects_raw(&cards, Zone::Library);
        let returned: Vec<_> = events
            .iter()
            .filter(|e| e.moved && e.destination == Zone::Library)
            .map(|e| e.after.card)
            .collect();
        let library = &mut self.state.players[player.index()].library;
        library.retain(|id| !returned.contains(id));
        library.splice(0..0, returned);
        self.finish_zone_events(&events, &[]);
    }
}
