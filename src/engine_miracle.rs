//! First-draw revelation and deferred draws are private, owned transactions.
use super::*;
use crate::cards::StaticAbilityFragment;
use std::collections::VecDeque;
#[derive(Clone, Debug, Serialize)]
pub struct MiracleChoiceView {
    pub object: ObjectRef,
    pub card: Card,
    pub can_reveal: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct RevealChoice {
    player: Player,
    object: ObjectRef,
    grant: Option<(AttachmentTriggerView, u16)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct DrawRequest {
    player: Player,
    amount: u128,
    replaced: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Protection {
    objects: Vec<CardId>,
    source: ObjectRef,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct MiracleState {
    choice: Option<RevealChoice>,
    draws: VecDeque<DrawRequest>,
    protection: Vec<Protection>,
}
pub(super) fn owned_bytes(data: &Option<Box<MiracleState>>) -> usize {
    data.as_ref().map_or(0, |s| {
        std::mem::size_of::<MiracleState>()
            + s.draws.len() * std::mem::size_of::<DrawRequest>()
            + s.protection
                .iter()
                .map(|r| {
                    std::mem::size_of::<Protection>()
                        + r.objects.len() * std::mem::size_of::<CardId>()
                })
                .sum::<usize>()
    })
}
impl Game {
    pub(super) fn miracle_cost_for(&self, item: &SpellView) -> Option<ManaCost> {
        item.cast
            .filter(|c| c.method == CastMethod::Miracle)
            .and_then(|_| {
                self.state
                    .resolution_free_casts
                    .as_ref()
                    .and_then(|f| f.alternative_cost)
            })
    }

    pub(super) fn miracle_choice_pending(&self) -> bool {
        self.state
            .miracle
            .as_ref()
            .is_some_and(|s| s.choice.is_some())
    }
    pub(super) fn miracle_choice_view(&self, viewer: Player) -> Option<MiracleChoiceView> {
        let choice = self.state.miracle.as_ref()?.choice.as_ref()?;
        if !self.shares_information(viewer, choice.player)
            || !matches!(self.state.pending, Pending::MiracleReveal { .. })
        {
            return None;
        }
        Some(MiracleChoiceView {
            object: choice.object,
            card: self.state.objects[choice.object.card as usize].card,
            can_reveal: choice.grant.is_some(),
        })
    }
    pub(super) fn offer_miracle_reveal(&mut self, player: Player, card: CardId) {
        if self.state.players[player.index()].cards_drawn_this_turn != 1 {
            return;
        }
        let object = self.object_ref(card);
        let definition = self.state.objects[card as usize].card;
        let mut provider = false;
        let mut grant = None;
        for &id in &self.state.static_sources {
            let source = self.object_view(id);
            if source.zone != Zone::Battlefield
                || source.controller != player
                || !source.printed_abilities_available()
            {
                continue;
            }
            for group in source.card.static_ability_groups() {
                for fragment in group.fragments {
                    if let StaticAbilityFragment::HandMiracle { filter, ability } = *fragment {
                        provider = true;
                        if grant.is_none()
                            && filter.matches(definition)
                            && self.state.objects[card as usize].zone == Zone::Hand
                        {
                            grant = Some((
                                AttachmentTriggerView {
                                    grantor: source.reference,
                                    granting_card: source.card,
                                    recipient: object,
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
        }
        // The same first-draw decision exists for an ineligible hidden card.
        if provider {
            self.state.miracle.get_or_insert_default().choice = Some(RevealChoice {
                player,
                object,
                grant,
            });
            self.state.pending = Pending::MiracleReveal { player };
        }
    }
    pub(super) fn draw_many_interruptible(&mut self, player: Player, amount: u128) {
        self.draw_requests(player, amount, false);
    }
    fn draw_requests(&mut self, player: Player, amount: u128, replaced: bool) {
        let attempts = amount.min(self.state.players[player.index()].library.len() as u128 + 1);
        if self.miracle_choice_pending() {
            self.state
                .miracle
                .as_mut()
                .unwrap()
                .draws
                .push_back(DrawRequest {
                    player,
                    amount: attempts,
                    replaced,
                });
            return;
        }
        for index in 0..attempts {
            let count = if replaced {
                1
            } else {
                self.replaced_draw_count(player)
            };
            let count = count.min(self.state.players[player.index()].library.len() as u128 + 1);
            for child in 0..count {
                self.draw_one_unreplaced(player);
                if self.miracle_choice_pending() {
                    let remaining = attempts - index - 1;
                    if remaining > 0 {
                        self.state
                            .miracle
                            .as_mut()
                            .unwrap()
                            .draws
                            .push_front(DrawRequest {
                                player,
                                amount: remaining,
                                replaced,
                            });
                    }
                    let children = count - child - 1;
                    if children > 0 {
                        self.state
                            .miracle
                            .as_mut()
                            .unwrap()
                            .draws
                            .push_front(DrawRequest {
                                player,
                                amount: children,
                                replaced: true,
                            });
                    }
                    return;
                }
            }
        }
    }
    pub(super) fn defer_draw_protection(
        &mut self,
        objects: Vec<CardId>,
        source: ObjectRef,
    ) -> bool {
        if !self.miracle_choice_pending() {
            return false;
        }
        self.state
            .miracle
            .as_mut()
            .unwrap()
            .protection
            .push(Protection { objects, source });
        true
    }
    pub(super) fn choose_miracle_reveal(&mut self, reveal: bool) {
        let choice = self.state.miracle.as_mut().unwrap().choice.take().unwrap();
        if reveal {
            let (grant, ability) = choice.grant.unwrap();
            if !self.state.known_hand.contains(&choice.object) {
                self.state.known_hand.push(choice.object);
            }
            let mut source = self.cast_preview(
                choice.player,
                choice.object.card,
                Zone::Hand,
                choice.object,
                0,
                [0; 6],
            );
            source.cast = None;
            self.queue_granted_spell_trigger(&source, grant, ability);
        }
        loop {
            let next = self.state.miracle.as_mut().unwrap().draws.pop_front();
            let Some(next) = next else {
                break;
            };
            self.draw_requests(next.player, next.amount, next.replaced);
            if self.miracle_choice_pending() {
                return;
            }
        }
        let state = self.state.miracle.take().unwrap();
        for row in state.protection {
            self.grant_keyword_effects(&row.objects, row.source, Keywords::INDESTRUCTIBLE);
        }
        if self.state.resolving.is_some() {
            self.run_program();
        } else {
            self.give_priority();
        }
    }
    pub(super) fn offer_miracle_cast(&mut self, item: &SpellView, cost: ManaCost) -> bool {
        if self.object_ref(item.card) != item.object
            || self.state.objects[item.card as usize].zone != Zone::Hand
        {
            return false;
        }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            casts_remaining: None,
            pending_cast: None,
            alternative_cost: Some(cost),
            maximum_mana_value: None,
            origin: Zone::Hand,
            exile_on_graveyard: false,
            allow_lands: false,
            player: item.controller,
            remaining: vec![item.object],
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }
}
