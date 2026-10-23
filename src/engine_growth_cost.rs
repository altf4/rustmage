//! Direct typed persistent evolution and non-targeting growth-card costs.
use super::*;
impl TriggerCondition {
    pub(super) fn is_growth_damage_event(self) -> bool {
        matches!(
            self,
            Self::SourceDealtCombatDamage
                | Self::PlayerDamaged {
                    subject: DamageSubject::Source,
                    combat_only: true,
                    opponent_only: false,
                    one_or_more: false
                }
        )
    }
}
impl Game {
    pub(super) fn evolve_source(
        &mut self,
        item: &SpellView,
        required: &'static str,
        types: &'static [&'static str],
        stats: Option<Stats>,
        keywords: Keywords,
        ability: Option<u16>,
    ) {
        if !self.on_battlefield(item.card)
            || self.object_ref(item.card) != item.object
            || !self.has_subtype(item.card, required)
        {
            return;
        }
        self.set_creature_types(item, Subject::Source, types);
        if let Some(stats) = stats {
            self.set_base_stats(
                item,
                Subject::Source,
                Some(stats.power),
                Some(stats.toughness),
            );
            self.state
                .roles
                .as_mut()
                .unwrap()
                .base_stats
                .last_mut()
                .unwrap()
                .permanent = true;
        }
        if keywords != Keywords::NONE {
            self.grant_keyword_effects(&[item.card], item.object, keywords);
            self.state
                .roles
                .as_mut()
                .unwrap()
                .keyword_effects
                .last_mut()
                .unwrap()
                .expires_turn = None;
        }
        if let Some(ability) = ability {
            let timestamp = self.next_role_timestamp();
            self.state
                .temporary_abilities
                .get_or_insert_with(|| Box::new(Vec::new()))
                .push(TemporaryAbilityView {
                    recipient: item.object,
                    grantor: item.object,
                    granting_card: item.source_card,
                    ability,
                    grantor_controller: item.controller,
                    expires_turn: self.state.turn,
                    permanent: true,
                    timestamp,
                });
        }
    }
}

impl Game {
    pub(super) fn attached_tap_spec() -> AdditionalCost {
        AdditionalCost {
            kind: EffectChoiceKind::TapPermanents(1),
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }
    }
    pub(super) fn visit_attached_activations(
        &self,
        card: CardId,
        actor: Player,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let grantor = self.object_ref(card);
        let giver = &self.state.objects[card as usize];
        let Some(recipient) = self.attachment_recipient(card) else {
            return false;
        };
        let source = self.object_view(recipient.card);
        for (index, ability) in giver.card.activations().iter().enumerate() {
            if ability_world::CharacteristicWorld::activation_prohibited(
                self,
                actor,
                recipient.card,
            ) || !giver.card.grants_activation(index as u16)
                || !source.grant_available(grantor, giver.card, index as u16, None)
                || (ability.mana_production().is_none()
                    && self.named_activation_prohibited(recipient.card))
                || !Self::object_activation_available_with_kinds(
                    &self.state.objects[recipient.card as usize],
                    self.counter_amounts(recipient.card),
                    self.current_kinds(recipient.card),
                    || self.keywords(recipient.card),
                    self.state.players[actor.index()].life,
                    ability,
                    index,
                    actor,
                    false,
                )
                || !self.additional_candidate(Self::attached_tap_spec(), actor, recipient, grantor)
            {
                continue;
            }
            if self.can_fund(FundingGoal {
                actor,
                required: ability.cost.colored,
                generic: ability.cost.generic,
                reserve: Some((grantor, Self::attached_tap_spec())),
                source: Some((recipient, index as u16, Counters::default(), giver.card)),
            }) && emit(Action::AnnounceAttachedAbility {
                grantor,
                ability: index as u16,
            }) {
                return true;
            }
        }
        false
    }
    pub(super) fn announce_attached_ability(&mut self, grantor: ObjectRef, ability: u16) {
        let recipient = self.attachment_recipient(grantor.card).unwrap();
        let giver = self.state.objects[grantor.card as usize].card;
        let actor = self.state.objects[recipient.card as usize].controller;
        let mut item = self.activation_item(actor, recipient.card, ability);
        item.source_card = giver;
        item.attachment_trigger = Some(Box::new(AttachmentTriggerView {
            grantor,
            granting_card: giver,
            recipient,
            defender: None,
            granted: true,
        }));
        self.open_payment_announcement(item, Counters::default(), None);
    }
    pub(super) fn attached_untap_triggers(&mut self, untapped: &[ObjectRef]) {
        let sources = self.state.attachment_sources.clone();
        for id in sources {
            if !self
                .attachment_recipient(id)
                .is_some_and(|r| untapped.contains(&r))
            {
                continue;
            }
            let source = self.object_view(id);
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::AttachedBecomesUntapped {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
}

impl Game {
    pub(super) fn creature_counter_spec(amount: u16) -> AdditionalCost {
        AdditionalCost {
            kind: EffectChoiceKind::RemoveCreatureCounters(amount),
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }
    }
    pub(super) fn visit_creature_counter_cost(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let amount = a.item.source_card.graveyard_counter_cost().unwrap();
        let mut selected = a
            .creature_counter_cost
            .as_ref()
            .map_or_else(Vec::new, |v| v.as_ref().clone());
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            for kind in CounterKind::ALL {
                if self.counter_count(r.card, kind) == 0 {
                    continue;
                }
                selected.push((r, kind));
                let available = self.creature_counter_fundable(
                    self.funding_goal(a),
                    self.purpose_for_item(&a.item),
                    amount,
                    &selected,
                );
                selected.pop();
                if available
                    && emit(Action::ChooseCreatureCounterCost {
                        object: r,
                        counter: kind,
                    })
                {
                    return true;
                }
            }
        }
        false
    }
    pub(super) fn choose_creature_counter_cost(&mut self, object: ObjectRef, kind: CounterKind) {
        let a = self.state.announcing.as_mut().unwrap();
        let amount = a.item.source_card.graveyard_counter_cost().unwrap();
        let selected = a
            .creature_counter_cost
            .get_or_insert_with(|| Box::new(Vec::new()));
        selected.push((object, kind));
        if selected.len() == usize::from(amount) {
            a.additional = Some(object);
            self.state.pending = Pending::PayCosts {
                player: a.item.controller,
            };
        }
    }
    pub(super) fn controlled_combat_damage_triggers(&mut self, events: &[DamageEvent]) {
        let sources = self.state.damage_sources.clone();
        for id in sources {
            let source = self.object_view(id);
            if source.controller != self.state.active {
                continue;
            }
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event()
                    != TriggerCondition::ControlledCreatureCombatDamageOwnTurn
                {
                    continue;
                }
                let mut amounts = std::collections::BTreeMap::<ObjectRef, u128>::new();
                for event in events {
                    if event.combat
                        && event.amount > 0
                        && event.source.controller == source.controller
                        && event.source.facts.is_creature()
                        && event.source.facts.battlefield
                    {
                        let n = amounts.entry(event.source.reference).or_default();
                        *n = n
                            .checked_add(event.amount as u128)
                            .capacity("damage amount exhausted");
                    }
                }
                for (dealer, amount) in amounts {
                    if self.queue_trigger(&source, index) {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .creature_combat_event =
                            Some(Box::new(CreatureCombatEventView { dealer, amount }));
                    }
                }
            }
        }
    }
}
