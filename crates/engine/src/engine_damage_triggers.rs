//! Damage observers and frozen amounts; no source graph is cloned.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn draw_many(&mut self, player: Player, amount: u128) {
        self.draw_many_interruptible(player, amount);
    }

    pub(super) fn is_modified(&self, id: CardId) -> bool {
        Self::object_modified_with(
            self.object_ref(id),
            &self.state.objects[id as usize],
            self.counter_amounts(id).has_any(),
            self.state
                .attachment_sources
                .iter()
                .map(|&id| &self.state.objects[id as usize]),
        )
    }

    pub(super) fn object_modified_with<'a>(
        source: ObjectRef,
        object: &Object,
        has_counters: bool,
        attachments: impl Iterator<Item = &'a Object>,
    ) -> bool {
        object.zone == Zone::Battlefield
            && (has_counters
                || attachments.into_iter().any(|attachment| {
                    attachment.zone == Zone::Battlefield
                        && attachment.attached_to == Some(source)
                        && (attachment.card.equipment().is_some()
                            || (attachment.card.aura().is_some()
                                && attachment.controller == object.controller))
                }))
    }

    pub(super) fn damage_facts(&self, id: CardId) -> DamageFacts {
        let object = &self.state.objects[id as usize];
        let creature = object.zone == Zone::Battlefield && self.is_creature(id);
        let (base, stats) = if creature {
            self.layered_stats(id)
        } else {
            (Stats::default(), Stats::default())
        };
        DamageFacts {
            token: object.is_token(),
            controller: object.controller,
            kinds: self.current_kinds(id),
            colors: ability_world::CharacteristicWorld::colors(self, id),
            power: stats.power,
            power_above_base: creature && stats.power > base.power,
            legendary_creature: creature && ability_world::CharacteristicWorld::legendary(self, id),
            granted_subtypes: self.granted_subtypes(id),
            card: if object.zone == Zone::Stack {
                self.effective_spell_card(id)
            } else {
                object.card
            },
            battlefield: object.zone == Zone::Battlefield,
            counter: self.counter_amounts(id).has_any(),
            plus_one: object.counters.plus_one > 0,
            modified: self.is_modified(id),
            attacking: self.state.attackers.iter().any(|a| a.card == id),
        }
    }

    fn damage_qualified(
        &self,
        source: &ObjectView,
        dealer: DamageSource,
        subject: DamageSubject,
    ) -> bool {
        match subject {
            DamageSubject::AnySources | DamageSubject::AnyNoncombatSources => true,
            DamageSubject::ControlledNoncombatSources => dealer.controller == source.controller,
            DamageSubject::ControlledSubtypes(subtypes) => {
                dealer.facts.battlefield
                    && dealer.controller == source.controller
                    && subtypes
                        .iter()
                        .any(|subtype| dealer.facts.has_subtype(subtype))
            }
            DamageSubject::Source => dealer.reference == source.reference,
            DamageSubject::Attached => {
                source.attached_to == Some(dealer.reference)
                    && self.on_battlefield(dealer.object)
                    && self.object_ref(dealer.object) == dealer.reference
            }
            DamageSubject::Creatures {
                filter,
                qualification,
            } => {
                let facts = dealer.facts;
                facts.battlefield
                    && facts.is_creature()
                    && match filter.relation {
                        0 => dealer.controller == source.controller,
                        1 => dealer.controller != source.controller,
                        _ => true,
                    }
                    && (!filter.other || dealer.reference != source.reference)
                    && filter
                        .subtype
                        .is_none_or(|subtype| facts.has_subtype(subtype))
                    && dealer.keywords.contains(filter.keyword)
                    && (!filter.attacking || facts.attacking)
                    && filter
                        .tokens
                        .is_none_or(|tokens| facts.is_token() == tokens)
                    && (!filter.artifact || facts.kinds & 16 != 0)
                    && match qualification {
                        DamageQualification::Any => true,
                        DamageQualification::PowerAboveBase => facts.power_above_base,
                        DamageQualification::Counter => facts.counter,
                        DamageQualification::PlusOne => facts.plus_one,
                        DamageQualification::Modified => facts.modified,
                    }
            }
        }
    }

    /// CR 120.4b: capture actual damage after prevention and before its results.
    pub(super) fn source_damage_triggers(&mut self, events: &[DamageEvent]) {
        if self.state.damage_sources.is_empty() {
            return;
        }
        let mut hits = std::collections::BTreeMap::<ObjectRef, u128>::new();
        for event in events {
            let reference = event.source.reference;
            if event.amount <= 0
                || !self.on_battlefield(reference.card)
                || self.object_ref(reference.card) != reference
                || !self.state.objects[reference.card as usize]
                    .card
                    .triggers()
                    .iter()
                    .any(|ability| ability.condition.event() == TriggerCondition::SourceDealtDamage)
            {
                continue;
            }
            let total = hits.entry(reference).or_default();
            *total = total
                .checked_add(event.amount as u128)
                .capacity("damage dealt amount exhausted");
        }
        for (reference, amount) in hits {
            let source = self.object_view(reference.card);
            for (index, ability) in source.card.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::SourceDealtDamage
                    && self.queue_trigger(&source, index)
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .set_event_object(EventObjectView {
                            object: reference,
                            facts: EventObjectFacts::DamageDealt { amount },
                            semantic_objects: None,
                            conniver_creature: None,
                            subtype_lki: None,
                        });
                }
            }
        }
    }

    pub(super) fn player_damage_triggers(&mut self, events: &[DamageEvent]) {
        if self.state.damage_sources.is_empty() {
            return;
        }
        // One dealer damaging one player simultaneously is one occurrence,
        // regardless of the number of assignment fragments in the event batch.
        let mut hits = std::collections::BTreeMap::new();
        for event in events {
            if event.amount <= 0 {
                continue;
            }
            let Target::Player(player) = event.target else {
                continue;
            };
            let amount = hits
                .entry((event.source, player, event.combat))
                .or_insert(0u128);
            *amount = amount
                .checked_add(u128::try_from(event.amount).expect("positive damage"))
                .capacity("damage amount exhausted");
        }
        let sources = self.indexed_trigger_sources(&self.state.damage_sources);
        for (source, grant) in sources {
            let definition = grant.as_ref().map_or(source.card, |g| g.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                let TriggerCondition::PlayerDamaged {
                    subject,
                    combat_only,
                    opponent_only,
                    one_or_more,
                } = ability.condition.event()
                else {
                    continue;
                };
                let mut aggregate = std::collections::BTreeMap::new();
                let names_dealer = !matches!(
                    subject,
                    DamageSubject::AnySources | DamageSubject::AnyNoncombatSources
                );
                for (&(dealer, player, combat), &amount) in &hits {
                    if (matches!(
                        subject,
                        DamageSubject::AnyNoncombatSources
                            | DamageSubject::ControlledNoncombatSources
                    ) && combat)
                        || (combat_only && !combat)
                        || (opponent_only && source.controller == player)
                        || !self.damage_qualified(&source, dealer, subject)
                    {
                        continue;
                    }
                    if one_or_more {
                        let total = aggregate.entry(player).or_insert((0u128, true, Vec::new()));
                        total.0 = total
                            .0
                            .checked_add(amount)
                            .capacity("damage amount exhausted");
                        total.1 &= combat;
                        if names_dealer && !total.2.contains(&dealer.reference) {
                            total.2.push(dealer.reference);
                        }
                    } else {
                        self.queue_damage_trigger(
                            &source,
                            index,
                            grant,
                            DamageTriggerView {
                                dealer: Some(dealer.reference),
                                player,
                                amount,
                                combat,
                            },
                            Some(dealer.controller),
                            if names_dealer {
                                vec![dealer.reference]
                            } else {
                                Vec::new()
                            },
                        );
                    }
                }
                for (player, (amount, combat, objects)) in aggregate {
                    self.queue_damage_trigger(
                        &source,
                        index,
                        grant,
                        DamageTriggerView {
                            dealer: None,
                            player,
                            amount,
                            combat,
                        },
                        None,
                        objects,
                    );
                }
            }
        }
    }

    /// Observe the completed damage event before state-based actions. In
    /// particular, post-event continuous effects qualify both observer and victim.
    pub(super) fn creature_received_damage_triggers(&mut self, events: &[DamageEvent]) {
        if !self.state.damage_sources.iter().any(|&id| {
            self.state.objects[id as usize]
                .card
                .triggers()
                .iter()
                .any(|ability| {
                    matches!(
                        ability.condition.event(),
                        TriggerCondition::CreatureDamaged { .. }
                            | TriggerCondition::SelfDamaged
                            | TriggerCondition::AttachedCreatureDamaged
                    )
                })
        }) {
            return;
        }
        let mut hits = std::collections::BTreeMap::new();
        for event in events {
            let Target::Card(id) = event.target else {
                continue;
            };
            if event.amount <= 0 || !self.on_battlefield(id) {
                continue;
            }
            let amount = hits.entry(self.object_ref(id)).or_insert(0u128);
            *amount = amount
                .checked_add(u128::try_from(event.amount).expect("positive damage"))
                .capacity("damage-received amount exhausted");
        }
        if hits.is_empty() {
            return;
        }
        let sources = self.indexed_trigger_sources(&self.state.damage_sources);
        for (source, grant) in sources {
            let definition = grant.as_ref().map_or(source.card, |g| g.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                let condition = ability.condition.event();
                if !matches!(
                    condition,
                    TriggerCondition::CreatureDamaged { .. }
                        | TriggerCondition::SelfDamaged
                        | TriggerCondition::AttachedCreatureDamaged
                ) {
                    continue;
                }
                for (&recipient, &amount) in &hits {
                    let controller = self.state.objects[recipient.card as usize].controller;
                    if !match condition {
                        TriggerCondition::SelfDamaged => recipient == source.reference,
                        TriggerCondition::AttachedCreatureDamaged => {
                            source.attached_to == Some(recipient)
                                && self.is_creature(recipient.card)
                                && self.attachment_recipient(source.id) == Some(recipient)
                        }
                        TriggerCondition::CreatureDamaged { relation } => {
                            self.is_creature(recipient.card)
                                && match relation {
                                    CastRelation::You => controller == source.controller,
                                    CastRelation::Opponent => controller != source.controller,
                                    CastRelation::Any => true,
                                }
                        }
                        _ => unreachable!("damage recipient condition"),
                    } {
                        continue;
                    }
                    if self.queue_trigger_from(&source, index, grant.map(Box::new)) {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .damage_received =
                            Some(Box::new(DamageReceivedView { recipient, amount }));
                    }
                }
            }
        }
    }

    fn queue_damage_trigger(
        &mut self,
        source: &ObjectView,
        index: usize,
        grant: Option<AttachmentTriggerView>,
        event: DamageTriggerView,
        dealer_controller: Option<Player>,
        semantic_dealers: Vec<ObjectRef>,
    ) {
        if !self.queue_trigger_from(source, index, grant.map(Box::new)) {
            return;
        }
        let packet = self.state.pending_triggers.last_mut().unwrap();
        let retain_dealer_controller = packet.trigger_definition().triggers()[index]
            .program
            .iter()
            .any(|instruction| {
                matches!(
                    instruction,
                    Instruction::ExileLibraryTopFaceDownPlay {
                        beneficiary: PlayerSet::EventPlayer,
                        ..
                    }
                )
            });
        packet.damage_event = Some(event);
        packet.refer_objects(semantic_dealers);
        if retain_dealer_controller {
            packet.event_player = dealer_controller;
        }
    }
}
