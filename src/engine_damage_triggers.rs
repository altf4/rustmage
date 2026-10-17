//! Player-damage observers and frozen amounts; no source graph is cloned.
use super::*;

impl Game {
    pub(super) fn draw_many(&mut self, player: Player, amount: u128) {
        // Empty draws only set the existing failed-draw flag. No admitted card
        // replaces draws or triggers on failed draws; cap the redundant attempts.
        let attempts = amount.min(self.state.players[player.index()].library.len() as u128 + 1);
        for _ in 0..attempts {
            self.draw(player);
        }
    }

    pub(super) fn is_modified(&self, id: CardId) -> bool {
        Self::object_modified_with(
            self.object_ref(id),
            &self.state.objects[id as usize],
            self.state
                .attachment_sources
                .iter()
                .map(|&id| &self.state.objects[id as usize]),
        )
    }

    pub(super) fn object_modified_with<'a>(
        source: ObjectRef,
        object: &Object,
        attachments: impl Iterator<Item = &'a Object>,
    ) -> bool {
        object.zone == Zone::Battlefield
            && (object.counters.plus_one > 0
                || object.counters.minus_one > 0
                || object.counters.stun > 0
                || object.counters.loyalty > 0
                || attachments.into_iter().any(|attachment| {
                    attachment.zone == Zone::Battlefield
                        && attachment.attached_to == Some(source)
                        && (attachment.card.equipment().is_some()
                            || (attachment.card.aura().is_some()
                                && attachment.owner == object.owner))
                }))
    }

    pub(super) fn damage_facts(&self, id: CardId) -> DamageFacts {
        let object = &self.state.objects[id as usize];
        DamageFacts {
            kinds: self.current_kinds(id),
            power: if object.zone == Zone::Battlefield && self.is_creature(id) {
                self.stats(id).power
            } else {
                0
            },
            granted_subtypes: self.granted_subtypes(id),
            card: if object.zone == Zone::Stack {
                self.effective_spell_card(id)
            } else {
                object.card
            },
            battlefield: object.zone == Zone::Battlefield,
            counter: object.counters.plus_one > 0
                || object.counters.minus_one > 0
                || object.counters.stun > 0
                || object.counters.loyalty > 0,
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
                        0 => dealer.controller == source.owner,
                        1 => dealer.controller != source.owner,
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
                        .is_none_or(|tokens| facts.card.is_token() == tokens)
                    && (!filter.artifact || facts.card.is_artifact())
                    && match qualification {
                        DamageQualification::Any => true,
                        DamageQualification::Counter => facts.counter,
                        DamageQualification::PlusOne => facts.plus_one,
                        DamageQualification::Modified => facts.modified,
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
                for (&(dealer, player, combat), &amount) in &hits {
                    if (combat_only && !combat)
                        || (opponent_only && source.owner == player)
                        || !self.damage_qualified(&source, dealer, subject)
                    {
                        continue;
                    }
                    if one_or_more {
                        let total = aggregate.entry(player).or_insert((0u128, true));
                        total.0 = total
                            .0
                            .checked_add(amount)
                            .capacity("damage amount exhausted");
                        total.1 &= combat;
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
                        );
                    }
                }
                for (player, (amount, combat)) in aggregate {
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
                    );
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
    ) {
        if !self.queue_trigger_from(source, index, grant.map(Box::new)) {
            return;
        }
        self.state.pending_triggers.last_mut().unwrap().damage_event = Some(event);
    }
}
