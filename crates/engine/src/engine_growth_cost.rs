//! Direct typed persistent evolution and non-targeting growth-card costs.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::StaticAbilityFragment;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TriggerCondition {
    pub(super) fn is_growth_damage_event(self) -> bool {
        matches!(
            self,
            Self::SourceDealtCombatDamage
                | Self::SourceCombatDamagePlayerOrPlaneswalker
                | Self::PlayerDamaged {
                    subject: DamageSubject::Source,
                    combat_only: true,
                    opponent_only: false,
                    one_or_more: false
                }
        )
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn visit_source_grants(
        &self,
        card: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        let controller = if object.zone == Zone::Battlefield {
            object.controller
        } else {
            object.owner
        };
        if controller != actor
            || !object.card.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(
                        fragment,
                        StaticAbilityFragment::SourceMaxSpeedActivation(_)
                            | StaticAbilityFragment::SourceEntryModeActivation { .. }
                    )
                })
            })
            || ability_world::CharacteristicWorld::activation_prohibited(self, actor, card)
            || self.nonmana_actions_prohibited()
            || self.named_activation_prohibited(card)
        {
            return false;
        }
        let source = self.object_ref(card);
        for (index, ability) in object.card.activations().iter().enumerate() {
            let index = index as u16;
            // Indexed mana templates use the normal mana action path in both
            // priority and nested payments, preserving all their real costs.
            if ability.mana_production().is_some()
                || !CharacteristicWorld::source_activation_template_available(self, card, index)
            {
                continue;
            }
            let readiness = self.activation_readiness(actor, ability);
            if activations::pure_loyalty_placement_prohibited(self, source, &readiness)
                || (ability.cost.sacrifice && CharacteristicWorld::sacrifice_prohibited(self, card))
                || !self.activation_guard_available(ability.guard, card, actor)
                || !Self::object_activation_available_with_kinds(
                    object,
                    self.counter_amounts(card),
                    self.current_kinds(card),
                    || CharacteristicWorld::activation_keywords(self, card),
                    self.state.players[actor.index()].life,
                    &readiness,
                    usize::from(index),
                    actor,
                    self.loyalty_timing(card, ability, actor, sorcery),
                )
                || (!ability.declares_target_value_x()
                    && !self.targets_available(ability.targets, actor, source))
                || !self
                    .activation_mana_costs(card, object.card, index, 0)
                    .any(|cost| {
                        self.initial_activation_target_fundable(
                            FundingGoal {
                                reserve: None,
                                actor,
                                required: cost.colored,
                                generic: cost.generic,
                                source: Some((
                                    source,
                                    index,
                                    Counters::default(),
                                    object.card,
                                    None,
                                )),
                            },
                            ability.targets,
                        )
                    })
            {
                continue;
            }
            if emit(Action::AnnounceSourceGrant {
                source,
                ability: index,
            }) {
                return true;
            }
        }
        false
    }

    pub(super) fn visit_controlled_activation_grants(
        &self,
        card: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let kinds = self.current_kinds(card);
        if kinds & (16 | 64) == 0 {
            return false;
        }
        let source = self.object_ref(card);
        let mut view = None;
        for &provider in &self.state.static_sources {
            let giver = &self.state.objects[provider as usize];
            if giver.zone != Zone::Battlefield || giver.controller != actor {
                continue;
            }
            let grantor = self.object_ref(provider);
            for fragment in giver
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
            {
                let (index, action) = match *fragment {
                    StaticAbilityFragment::ControlledPlaneswalkerActivation(index)
                        if kinds & 64 != 0 =>
                    {
                        (
                            index,
                            Action::AnnouncePlaneswalkerGrant {
                                source,
                                grantor,
                                ability: index,
                            },
                        )
                    }
                    StaticAbilityFragment::ControlledArtifactFood(index) if kinds & 16 != 0 => (
                        index,
                        Action::AnnounceArtifactFood {
                            source,
                            grantor,
                            ability: index,
                        },
                    ),
                    _ => continue,
                };
                if CharacteristicWorld::activation_prohibited(self, actor, card)
                    || self.named_activation_prohibited(card)
                {
                    return false;
                }
                let ability = giver.card.activation(index);
                let Some(readiness) = self
                    .external_exhaust_readiness(source, grantor, giver.card, index, actor, ability)
                else {
                    continue;
                };
                if activations::pure_loyalty_placement_prohibited(self, source, &readiness)
                    || (ability.cost.sacrifice
                        && CharacteristicWorld::sacrifice_prohibited(self, card))
                    || !view
                        .get_or_insert_with(|| self.object_view(card))
                        .grant_available(grantor, giver.card, index, None)
                    || !self.activation_guard_available(ability.guard, card, actor)
                    || !Self::object_activation_available_with_kinds(
                        &self.state.objects[card as usize],
                        self.counter_amounts(card),
                        self.current_kinds(card),
                        || ability_world::CharacteristicWorld::activation_keywords(self, card),
                        self.state.players[actor.index()].life,
                        &readiness,
                        usize::from(index),
                        actor,
                        self.loyalty_timing(card, ability, actor, sorcery),
                    )
                    || (!ability.declares_target_value_x()
                        && !self.targets_available(ability.targets, actor, source))
                    || !self
                        .activation_mana_costs(card, giver.card, index, 0)
                        .any(|cost| {
                            self.initial_activation_target_fundable(
                                FundingGoal {
                                    reserve: None,
                                    actor,
                                    required: cost.colored,
                                    generic: cost.generic,
                                    source: Some((
                                        source,
                                        index,
                                        Counters::default(),
                                        giver.card,
                                        None,
                                    )),
                                },
                                ability.targets,
                            )
                        })
                {
                    continue;
                }
                if emit(action) {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn announce_controlled_activation_grant(
        &mut self,
        source: ObjectRef,
        grantor: ObjectRef,
        ability: u16,
    ) {
        let giver = self.state.objects[grantor.card as usize].card;
        let object = &self.state.objects[source.card as usize];
        let actor = if object.zone == Zone::Battlefield {
            object.controller
        } else {
            object.owner
        };
        let mut item = self.activation_item(actor, source.card, ability);
        item.set_source_definition(giver);
        item.source_ability_origin = self.copied_grant_origin(grantor);
        item.attachment_trigger = Some(Box::new(AttachmentTriggerView {
            grantor,
            granting_card: giver,
            recipient: source,
            defender: None,
            granted: true,
            origin: item.source_ability_origin,
        }));
        self.open_payment_announcement(item, Counters::default(), None);
    }
    pub(super) fn evolve_source(
        &mut self,
        item: &SpellView,
        required: &'static str,
        types: &'static [&'static str],
        stats: Option<Stats>,
        keywords: Keywords,
        ability: Option<u16>,
    ) {
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.id)
            || self.object_ref(source.id) != source.object
            || !self.has_subtype(source.id, required)
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
            self.grant_keyword_effects(&[source.id], source.object, keywords);
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
                    recipient: source.object,
                    grantor: source.object,
                    granting_card: source.card,
                    ability,
                    grantor_controller: item.controller,
                    expires_turn: self.state.turn,
                    permanent: true,
                    timestamp,
                    origin: Some(timestamp),
                });
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn visit_attached_activations(
        &self,
        card: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let grantor = self.object_ref(card);
        let giver = &self.state.objects[card as usize];
        let Some(recipient) = self.attachment_recipient(card) else {
            return false;
        };
        let source = self.object_view(recipient.card);
        for (index, ability) in giver.card.activations().iter().enumerate() {
            let Some(readiness) = self.external_exhaust_readiness(
                recipient,
                grantor,
                giver.card,
                index as u16,
                actor,
                ability,
            ) else {
                continue;
            };
            if ability_world::CharacteristicWorld::activation_prohibited(
                self,
                actor,
                recipient.card,
            ) || !giver.card.grants_activation(index as u16)
                || !source.grant_available(grantor, giver.card, index as u16, None)
                || (ability.mana_production().is_none()
                    && (self.nonmana_actions_prohibited()
                        || self.named_activation_prohibited(recipient.card)))
                || (ability.cost.sacrifice
                    && CharacteristicWorld::sacrifice_prohibited(self, recipient.card))
                || !self.activation_guard_available(ability.guard, recipient.card, actor)
                || !Self::object_activation_available_with_kinds(
                    &self.state.objects[recipient.card as usize],
                    self.counter_amounts(recipient.card),
                    self.current_kinds(recipient.card),
                    || {
                        ability_world::CharacteristicWorld::activation_keywords(
                            self,
                            recipient.card,
                        )
                    },
                    self.state.players[actor.index()].life,
                    &readiness,
                    index,
                    actor,
                    self.loyalty_timing(recipient.card, ability, actor, sorcery),
                )
            {
                continue;
            }
            let item = self.attached_activation_packet(actor, recipient, grantor, index as u16);
            if (ability.declares_target_value_x()
                || self.item_targets_available(&item, ability.targets))
                && self.initial_activation_packet_target_fundable(&item, &Counters::default())
                && emit(Action::AnnounceAttachedAbility {
                    grantor,
                    ability: index as u16,
                })
            {
                return true;
            }
        }
        false
    }
    pub(super) fn announce_attached_ability(&mut self, grantor: ObjectRef, ability: u16) {
        let recipient = self.attachment_recipient(grantor.card).unwrap();
        let actor = self.state.objects[recipient.card as usize].controller;
        let mut item = self.attached_activation_packet(actor, recipient, grantor, ability);
        item.id = self.next_stack_id();
        self.state.passes = 0;
        self.open_payment_announcement(item, Counters::default(), None);
    }
    fn attached_activation_packet(
        &self,
        actor: Player,
        recipient: ObjectRef,
        grantor: ObjectRef,
        ability: u16,
    ) -> SpellView {
        let giver = self.state.objects[grantor.card as usize].card;
        let mut item = self.activation_packet(actor, recipient.card, ability);
        item.set_source_definition(giver);
        item.source_ability_origin = self.copied_grant_origin(grantor);
        item.attachment_trigger = Some(Box::new(AttachmentTriggerView {
            grantor,
            granting_card: giver,
            recipient,
            defender: None,
            granted: true,
            origin: item.source_ability_origin,
        }));
        item
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
                if ability.condition.event() == TriggerCondition::AttachedBecomesUntapped
                    && let Some(recipient) = source.attached_to
                    && self.queue_trigger(&source, index)
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .refer_objects([recipient]);
                }
            }
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
    pub(super) fn visit_distributed_counter_cost(
        &self,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let spec = a.counter_cost_spec().unwrap();
        let requirement = spec.kind.counter_requirement().unwrap();
        let mut selected = a
            .creature_counter_cost
            .as_ref()
            .map_or_else(Vec::new, |v| v.as_ref().clone());
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if !additional_costs::counter_cost_bearer(
                self,
                a.item.controller,
                r,
                requirement.bearer_kinds,
            ) {
                continue;
            }
            for kind in CounterKind::ALL {
                if requirement.kind.is_some_and(|expected| expected != kind)
                    || self.counter_count(r.card, kind) == 0
                {
                    continue;
                }
                selected.push((r, kind));
                let available = self.counter_cost_fundable(
                    self.funding_goal(a),
                    self.purpose_for_item(&a.item),
                    spec,
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
    pub(super) fn open_counter_cost_mana_if_needed(&mut self) {
        let Pending::AdditionalCost { player } = self.state.pending else {
            return;
        };
        let a = self.state.announcing.as_ref().unwrap();
        if a.cast_costs.is_some()
            || a.counter_cost_spec().is_none()
            || self.current_counter_cost_payable(a)
        {
            return;
        }
        self.open_counter_cost_mana(player);
    }
    pub(super) fn open_counter_cost_mana(&mut self, player: Player) {
        // Mana abilities may produce a missing counter or return its bearer.
        // Open a mana window before selecting the ordered counter payment.
        let a = self.state.announcing.as_mut().unwrap();
        a.add_cast_cost_clause(CastCostClause::Mana);
        a.add_cast_cost_clause(CastCostClause::Additional);
        self.state.pending = Pending::PayCosts { player };
    }
    pub(super) fn choose_distributed_counter_cost(&mut self, object: ObjectRef, kind: CounterKind) {
        let a = self.state.announcing.as_mut().unwrap();
        let requirement = a
            .counter_cost_spec()
            .unwrap()
            .kind
            .counter_requirement()
            .unwrap();
        let selected = a
            .creature_counter_cost
            .get_or_insert_with(|| Box::new(Vec::new()));
        selected.push((object, kind));
        if selected.len() == usize::from(requirement.amount) {
            a.additional = Some(object);
            self.state.pending = Pending::PayCosts {
                player: a.item.controller,
            };
            self.resume_cast_cost_selection();
        } else {
            self.open_counter_cost_mana_if_needed();
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

// Grant provenance binds printed costs to the same permanent that gave the
// ability (CR 201.5a), even after an intervening mana ability ends the grant.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellView {
    fn attached_activation_grant(&self) -> Option<(&AttachmentTriggerView, u16)> {
        let StackKind::ActivatedAbility { ability } = self.kind else {
            return None;
        };
        let grant = self.attachment_trigger.as_deref()?;
        (grant.granted && grant.grantor != self.source?.object).then_some((grant, ability))
    }

    pub(super) fn attached_activation_tap_cost(&self) -> Option<ObjectRef> {
        let (grant, ability) = self.attached_activation_grant()?;
        grant
            .granting_card
            .attached_activation_taps(ability)
            .then_some(grant.grantor)
    }

    pub(super) fn attached_activation_exile_cost(&self) -> Option<ObjectRef> {
        let (grant, ability) = self.attached_activation_grant()?;
        grant
            .granting_card
            .attached_activation_exiles(ability)
            .then_some(grant.grantor)
    }

    pub(super) fn attached_activation_reserve(&self) -> Option<(ObjectRef, AdditionalCost)> {
        let (grantor, kind) = if let Some(grantor) = self.attached_activation_tap_cost() {
            (grantor, EffectChoiceKind::TapPermanents(1))
        } else {
            (
                self.attached_activation_exile_cost()?,
                EffectChoiceKind::ExileGrantingAttachment,
            )
        };
        Some((
            grantor,
            AdditionalCost {
                kind,
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
            },
        ))
    }
}
