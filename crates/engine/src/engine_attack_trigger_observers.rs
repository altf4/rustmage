//! Observers of actual abilities triggered by a creature's own attack.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellView {
    pub(super) fn observes_trigger(&self) -> bool {
        self.trigger_event() == Some(TriggerCondition::OwnAttackerAbilityTriggered)
    }

    fn trigger_event(&self) -> Option<TriggerCondition> {
        let StackKind::TriggeredAbility { ability } = self.kind else {
            return None;
        };
        self.source()?;
        self.trigger_definition()
            .triggers()
            .get(usize::from(ability))
            .map(|trigger| trigger.condition.event())
    }

    fn names_declared_attacker(&self, attacker: ObjectRef) -> bool {
        self.entering_object == Some(attacker)
            || self.reflexive_subject.as_ref().is_some_and(|subject| {
                subject.object == attacker || subject.additional_objects.contains(&attacker)
            })
            || self.event_object.as_ref().is_some_and(|event| {
                event.object == attacker
                    || event
                        .semantic_objects
                        .as_ref()
                        .is_some_and(|group| group.contains(&DamageSourceRef::Object(attacker)))
            })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn caused_by_own_attack(
        &self,
        item: &SpellView,
        source: ObjectRef,
        attack: &AttackDeclaration,
    ) -> bool {
        if item.kind == StackKind::DecayedAttack {
            return true;
        }
        let Some(event) = item.trigger_event() else {
            return false;
        };
        match event {
            TriggerCondition::SelfAttacks
            | TriggerCondition::SelfAttacksDefendingPlayer
            | TriggerCondition::SelfAttacksIfEquipped
            | TriggerCondition::SelfAttacksOrBlocks
            | TriggerCondition::SelfEntersOrAttacks
            | TriggerCondition::SelfFirstAttack
            | TriggerCondition::SelfAttacksUnlessDefenderControlsCreatureSubtype(_)
            | TriggerCondition::SelfAttacksPlayerWithMostLife
            | TriggerCondition::SelfAttacksAlone
            | TriggerCondition::SelfAttacksWithOthers
            | TriggerCondition::SelfAttacksWhileSaddled
            | TriggerCondition::SelfEntersOrAttacksWhileSaddled
            | TriggerCondition::GrantedAttacks => true,
            TriggerCondition::AttackDeclared { power_min, .. } => {
                power_min.is_none_or(|minimum| self.stats(source.card).power >= i128::from(minimum))
            }
            TriggerCondition::OpponentAttacked => {
                attack.defender == Some(CombatDefender::Player(item.controller.other()))
            }
            TriggerCondition::ControlledCreatureAttacks
            | TriggerCondition::CreatureAttacksYou { .. }
            | TriggerCondition::ControlledCreatureAttacksOrBlocks
            | TriggerCondition::ControlledCreatureAttacksWithKeyword(_)
            | TriggerCondition::ControlledCreatureWithKeywordAttackDeclaration(_)
            | TriggerCondition::ControlledCreatureAttacksWithCounter(_)
            | TriggerCondition::ControlledSubtypeAttacks { .. }
            | TriggerCondition::ControlledSubtypeAttacksAny(_)
            | TriggerCondition::ControlledSubtypeAttackDeclaration(_)
            | TriggerCondition::ControlledSubtypeAttackAtLeast { .. }
            | TriggerCondition::ControlledSuspectedCreaturesAttack
            | TriggerCondition::ControlledCreatureAttacksAlone
            | TriggerCondition::ControlledCreatureAttacksPlayerAlone
            | TriggerCondition::ControlledEquippedCreatureAttacks
            | TriggerCondition::CreaturesAttackDeclared { .. }
            | TriggerCondition::AttackPlayer(_)
            | TriggerCondition::AttackPlayerExcludingSubtype(_)
            | TriggerCondition::AttackPlayerWithPowerAtLeast(_)
            | TriggerCondition::FirstAttackWithTotalPowerAtLeast(_)
            | TriggerCondition::PlayerAttacksWithAtLeast(_)
            | TriggerCondition::CrewedVehicleAttacks => item.names_declared_attacker(source),
            TriggerCondition::AttachedAttacks
            | TriggerCondition::AttachedAttacksOrBlocks
            | TriggerCondition::EquippedAttacks
            | TriggerCondition::EquippedAttacksAlone => item
                .attachment_trigger
                .as_ref()
                .is_some_and(|event| event.recipient == source),
            // Registered delayed callbacks are not abilities of the attacker.
            _ => false,
        }
    }

    pub(super) fn observe_attack_triggers(
        &mut self,
        actor: Player,
        declared: &[AttackDeclaration],
        first_trigger: usize,
    ) {
        let Some(sources) = self.state.bend_sources.as_ref() else {
            return;
        };
        let observers: Vec<_> = sources
            .iter()
            .copied()
            .filter(|&id| {
                let object = &self.state.objects[id as usize];
                object.controller == actor
                    && self.has_trigger(id, |ability| {
                        ability.condition.event() == TriggerCondition::OwnAttackerAbilityTriggered
                    })
            })
            .map(|id| self.object_view(id))
            .filter(ObjectView::printed_abilities_available)
            .collect();
        if observers.is_empty() {
            return;
        }
        let mut causes = Vec::new();
        for index in first_trigger..self.state.pending_triggers.len() {
            let item = &self.state.pending_triggers[index];
            if item.is_copy || item.controller != actor {
                continue;
            }
            let Some(source) = item.source().map(|source| source.object) else {
                continue;
            };
            let Some(attack) = declared.iter().find(|attack| attack.card == source.card) else {
                continue;
            };
            if self.object_ref(source.card) != source
                || !self.is_creature(source.card)
                || !self.caused_by_own_attack(item, source, attack)
            {
                continue;
            }
            let item = &mut self.state.pending_triggers[index];
            causes.push(StackLink {
                id: item.id,
                controller: actor,
            });
            let extra = std::mem::take(&mut item.additional_triggers);
            if extra == 0 {
                continue;
            }
            let template = item.clone();
            for _ in 0..extra {
                let mut occurrence = template.clone();
                occurrence.id = self.next_stack_id();
                causes.push(StackLink {
                    id: occurrence.id,
                    controller: actor,
                });
                self.state.pending_triggers.push(occurrence);
            }
        }
        for cause in causes {
            for observer in &observers {
                for (index, ability) in observer.trigger_templates() {
                    if ability.condition.event() == TriggerCondition::OwnAttackerAbilityTriggered
                        && self.queue_trigger(observer, index)
                    {
                        self.state
                            .pending_triggers
                            .last_mut()
                            .unwrap()
                            .related_stack = Some(cause);
                    }
                }
            }
        }
    }

    pub(super) fn conditional_self_attack_triggers(&mut self, source: &ObjectView) {
        let Some(available) = &source.abilities else {
            return;
        };
        let granted = source
            .card
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .filter_map(|fragment| match *fragment {
                StaticAbilityFragment::SourceConditionalAttackTrigger { ability, .. } => {
                    Some(ability)
                }
                _ => None,
            });
        for ability in granted {
            if !available.grants.iter().any(|grant| {
                grant.grantor == source.reference
                    && grant.granting_card == source.card
                    && grant.ability == Some(ability)
                    && grant.static_rule.is_none()
            }) {
                continue;
            }
            self.queue_attack_trigger_from(
                source,
                usize::from(ability),
                Some(Box::new(AttachmentTriggerView {
                    grantor: source.reference,
                    granting_card: source.card,
                    recipient: source.reference,
                    defender: None,
                    granted: true,
                    origin: source.copied_grant_origin(),
                })),
            );
        }
    }
}
