//! First-draw revelation and deferred draws are private, owned transactions.
use super::*;
use crate::cards::StaticAbilityFragment;
use std::collections::VecDeque;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct MiracleChoiceView {
    pub object: ObjectRef,
    pub card: Card,
    pub can_reveal: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct RevealChoice {
    player: Player,
    object: ObjectRef,
    grant: Option<(AttachmentTriggerView, u16)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct DrawReplacementView {
    pub player: Player,
    pub additional_sources: Vec<ObjectRef>,
    pub doubling_sources: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct DrawOrder {
    request: DrawRequest,
    additional: Vec<ObjectRef>,
    doublers: Vec<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct DrawRequest {
    player: Player,
    amount: u128,
    replaced: bool,
    batch_checked: bool,
    excluded_doublers: Vec<ObjectRef>,
    /// Resolving spell which owns this request, if it remembers actual draws.
    remembering: Option<u64>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Protection {
    objects: Vec<CardId>,
    source: ObjectRef,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct MiracleState {
    choice: Option<RevealChoice>,
    draw_order: Option<DrawOrder>,
    remaining_choices: VecDeque<RevealChoice>,
    draws: VecDeque<DrawRequest>,
    protection: Vec<Protection>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(data: &Option<Box<MiracleState>>) -> usize {
    data.as_ref().map_or(0, |s| {
        std::mem::size_of::<MiracleState>()
            + s.remaining_choices.len() * std::mem::size_of::<RevealChoice>()
            + s.draws
                .iter()
                .map(|r| {
                    std::mem::size_of::<DrawRequest>()
                        + r.excluded_doublers.len() * std::mem::size_of::<ObjectRef>()
                })
                .sum::<usize>()
            + s.draw_order.as_ref().map_or(0, |r| {
                (r.additional.len() + r.doublers.len() + r.request.excluded_doublers.len())
                    * std::mem::size_of::<ObjectRef>()
            })
            + s.protection
                .iter()
                .map(|r| {
                    std::mem::size_of::<Protection>()
                        + r.objects.len() * std::mem::size_of::<CardId>()
                })
                .sum::<usize>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
            .is_some_and(|s| s.choice.is_some() || s.draw_order.is_some())
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
        let mut choices = VecDeque::new();
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
                        let grant = (filter.matches(definition)
                            && self.state.objects[card as usize].zone == Zone::Hand)
                            .then(|| {
                                (
                                    AttachmentTriggerView {
                                        grantor: source.reference,
                                        granting_card: source.card,
                                        recipient: object,
                                        defender: None,
                                        granted: true,
                                        origin: source.copied_grant_origin(),
                                    },
                                    ability,
                                )
                            });
                        // Keep one private slot per public grant, even when its
                        // filter rejects the hidden card. Otherwise the number
                        // of decisions would reveal that card's eligibility.
                        choices.push_back(RevealChoice {
                            player,
                            object,
                            grant,
                        });
                    }
                }
            }
        }
        if let Some(choice) = choices.pop_front() {
            let state = self.state.miracle.get_or_insert_default();
            debug_assert!(state.choice.is_none() && state.remaining_choices.is_empty());
            state.choice = Some(choice);
            state.remaining_choices = choices;
            self.state.pending = Pending::MiracleReveal { player };
        }
    }
    pub(super) fn draw_many_interruptible(&mut self, player: Player, amount: u128) {
        self.draw_requests(player, amount, false, None);
    }
    pub(super) fn draw_many_recording(&mut self, player: Player, amount: u128, item_id: u64) {
        self.draw_requests(player, amount, false, Some(item_id));
    }
    fn draw_requests(
        &mut self,
        player: Player,
        amount: u128,
        replaced: bool,
        remembering: Option<u64>,
    ) {
        self.perform_draw_request(DrawRequest {
            player,
            amount,
            replaced,
            remembering,
            batch_checked: false,
            excluded_doublers: Vec::new(),
        });
    }
    fn perform_draw_request(&mut self, mut request: DrawRequest) {
        if self.miracle_choice_pending() {
            self.state
                .miracle
                .as_mut()
                .unwrap()
                .draws
                .push_back(request);
            return;
        }
        let player = request.player;
        if !request.replaced && !request.batch_checked && request.amount > 0 {
            let additional = self.quantum_draw_sources(player);
            let doublers = if request.amount == 1 && !additional.is_empty() {
                self.doubled_draw_sources(player).collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            // CR121.2a/616.1g: a multi-draw instruction is modified before its
            // individual draws. For a single draw, the affected player can
            // choose which overlapping replacement changes the event first.
            if request.amount == 1 && !additional.is_empty() && !doublers.is_empty() {
                self.state.miracle.get_or_insert_default().draw_order = Some(DrawOrder {
                    request,
                    additional,
                    doublers,
                });
                self.state.pending = Pending::MiracleReveal { player };
                return;
            }
            request.amount = request
                .amount
                .checked_add(additional.len() as u128)
                .capacity("draw replacement quantity exhausted");
            request.batch_checked = true;
        }
        let attempts = request
            .amount
            .min(self.library_card_count(player) as u128 + 1);
        for index in 0..attempts {
            let count = if request.replaced {
                1
            } else {
                self.replaced_draw_count_excluding(player, &request.excluded_doublers)
            };
            let count = count.min(self.library_card_count(player) as u128 + 1);
            for child in 0..count {
                if self.draw_one_unreplaced(player)
                    && let Some(id) = request.remembering
                {
                    let resolving = self
                        .state
                        .resolving
                        .as_mut()
                        .expect("remembered draw must have its resolution");
                    assert_eq!(
                        resolving.item.id, id,
                        "draw request belongs to a different resolution"
                    );
                    let count = resolving
                        .remembered_draws
                        .as_mut()
                        .expect("remembered draw counter");
                    *count = count
                        .checked_add(1)
                        .capacity("remembered draw count exhausted");
                }
                if self.miracle_choice_pending() {
                    let remaining = attempts - index - 1;
                    if remaining > 0 {
                        self.state
                            .miracle
                            .as_mut()
                            .unwrap()
                            .draws
                            .push_front(DrawRequest {
                                amount: remaining,
                                ..request.clone()
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
                                amount: children,
                                replaced: true,
                                ..request.clone()
                            });
                    }
                    return;
                }
            }
        }
    }
    pub(super) fn draw_replacement_view(&self) -> Option<DrawReplacementView> {
        let order = self.state.miracle.as_ref()?.draw_order.as_ref()?;
        Some(DrawReplacementView {
            player: order.request.player,
            additional_sources: order.additional.clone(),
            doubling_sources: order.doublers.clone(),
        })
    }
    pub(super) fn choose_draw_replacement(&mut self, source: ObjectRef) {
        let order = self
            .state
            .miracle
            .as_mut()
            .unwrap()
            .draw_order
            .take()
            .unwrap();
        let mut request = order.request;
        if order.doublers.contains(&source) {
            request.amount = 2;
            request.excluded_doublers.push(source);
        }
        request.amount = request
            .amount
            .checked_add(order.additional.len() as u128)
            .capacity("draw replacement quantity exhausted");
        request.batch_checked = true;
        self.perform_draw_request(request);
        self.finish_pending_draws();
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
            self.remember_public_hand(choice.object);
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
        // Every grant gets its own immediate reveal choice. Do not resume the
        // enclosing draw or put triggers on the stack between these choices.
        let state = self.state.miracle.as_mut().unwrap();
        if let Some(next) = state.remaining_choices.pop_front() {
            state.choice = Some(next);
            return;
        }
        self.finish_pending_draws();
    }
    fn finish_pending_draws(&mut self) {
        if self.miracle_choice_pending() {
            return;
        }
        loop {
            let next = self.state.miracle.as_mut().unwrap().draws.pop_front();
            let Some(next) = next else {
                break;
            };
            self.perform_draw_request(next);
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
        let Some(source) = item.source else {
            return false;
        };
        if self.object_ref(source.id) != source.object
            || self.state.objects[source.id as usize].zone != Zone::Hand
        {
            return false;
        }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: None,
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: Some(cost),
            maximum_mana_value: None,
            origin: Zone::Hand,
            exile_on_graveyard: false,
            allow_lands: false,
            player: item.controller,
            remaining: vec![source.object],
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }
}
