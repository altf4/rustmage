//! Copiable immutable activation templates with exact linked-acquisition identity.
use super::*;
use crate::cards::RetainedCopyActivation;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellView {
    pub(super) fn is_retained_copy_activation(&self) -> bool {
        let StackKind::ActivatedAbility { ability } = self.kind else {
            return false;
        };
        self.source_copy_recipe
            .as_ref()
            .and_then(|r| r.exceptions.retained_activation)
            .is_some_and(|retained| {
                retained.ability == ability
                    && self.source.is_some_and(|s| s.card == retained.definition)
            })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn retained_link_origin(&self, source: ObjectRef) -> Option<u64> {
        self.temporary_copy_row(source)
            .and_then(|row| row.active().value.retained_link_origin)
            .map_or_else(
                || self.copy_ability_origin(source),
                |origin| origin.ability_origin,
            )
    }
    pub(super) fn linked_exile_for_origin(
        &self,
        source: ObjectRef,
        origin: Option<u64>,
    ) -> Vec<ObjectRef> {
        self.state
            .linked_choices
            .as_ref()
            .map_or_else(Vec::new, |state| {
                state
                    .links
                    .iter()
                    .filter(|link| {
                        link.source == source
                            && link.ability_origin == origin
                            && self.object_ref(link.exiled.card) == link.exiled
                            && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
                    })
                    .map(|link| link.exiled)
                    .collect()
            })
    }
    fn retained_copy_template(&self, source: ObjectRef) -> Option<RetainedCopyActivation> {
        if !self.on_battlefield(source.card) || self.object_ref(source.card) != source {
            return None;
        }
        self.copy_recipe(source)?.exceptions.retained_activation
    }
    pub(super) fn visit_retained_copy_activation(
        &self,
        id: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let source = self.object_ref(id);
        let Some(template) = self.retained_copy_template(source) else {
            return false;
        };
        let rules = template.definition.activation(template.ability);
        assert!(
            rules.origin == ActivationOrigin::Battlefield
                && !rules.once_per_turn
                && !rules.exhaust
                && rules.mana_production().is_none(),
            "unsupported retained activation lifecycle"
        );
        if self.nonmana_actions_prohibited()
            || self.state.objects[id as usize].controller != actor
            || ability_world::CharacteristicWorld::activation_prohibited(self, actor, id)
            || self.named_activation_prohibited(id)
            || !self.printed_abilities_available(id)
            || !self.activation_guard_available(rules.guard, id, actor)
        {
            return false;
        }
        let readiness = self.activation_readiness(actor, rules);
        if activations::pure_loyalty_placement_prohibited(self, source, &readiness)
            || (rules.cost.sacrifice
                && ability_world::CharacteristicWorld::sacrifice_prohibited(self, id))
            || !Self::object_activation_available_with_kinds(
                &self.state.objects[id as usize],
                self.counter_amounts(id),
                self.current_kinds(id),
                || ability_world::CharacteristicWorld::activation_keywords(self, id),
                self.state.players[actor.index()].life,
                &readiness,
                usize::from(template.ability),
                actor,
                self.loyalty_timing(id, rules, actor, sorcery),
            )
        {
            return false;
        }
        let mut targets = rules.targets;
        if let TargetPlan::Single(mut spec) = targets
            && matches!(spec.rule, TargetRule::LinkedExileCreature { .. })
        {
            spec.rule = TargetRule::LinkedExileCreature {
                ability_origin: Some(self.retained_link_origin(source)),
            };
            targets = TargetPlan::Single(spec);
        }
        (rules.declares_target_value_x() || self.targets_available(targets, actor, source))
            && self
                .activation_mana_costs(id, template.definition, template.ability, 0)
                .any(|cost| {
                    self.initial_activation_target_fundable(
                        FundingGoal {
                            actor,
                            required: cost.colored,
                            generic: cost.generic,
                            reserve: None,
                            source: Some((
                                source,
                                template.ability,
                                Counters::default(),
                                template.definition,
                                None,
                            )),
                        },
                        targets,
                    )
                })
            && emit(Action::AnnounceRetainedCopyAbility { source })
    }
    pub(super) fn announce_retained_copy_ability(&mut self, source: ObjectRef) {
        let template = self.retained_copy_template(source).unwrap();
        let actor = self.state.objects[source.card as usize].controller;
        let origin = self.retained_link_origin(source);
        let mut item = self.activation_item(actor, source.card, template.ability);
        item.set_source_definition(template.definition);
        item.source_ability_origin = origin;
        item.source_transform = None;
        self.bind_activation_source_quantities(
            &mut item,
            template.definition.activation(template.ability).program,
        );
        self.open_payment_announcement(item, Counters::default(), None);
    }
    pub(super) fn copy_source_from_linked_exile(&mut self, item: &SpellView, base: (i16, i16)) {
        let Some(source) = item.source else { return };
        if !self.on_battlefield(source.id) || self.object_ref(source.id) != source.object {
            return;
        }
        let Some(target) = item.targets.iter().find_map(|bound| {
            self.same_target_object(*bound)
                .then_some(bound.object)
                .flatten()
        }) else {
            return;
        };
        let object = &self.state.objects[target.card as usize];
        if object.zone != Zone::Exile
            || !object.is_card()
            || !object.card.is_creature()
            || !self
                .linked_exile_for_origin(source.object, item.source_ability_origin)
                .contains(&target)
        {
            return;
        }
        let StackKind::ActivatedAbility { ability } = item.kind else {
            return;
        };
        let recipe = CopyRecipe::new(object.card).with_exceptions(CopyExceptions {
            base: Some(base),
            retained_activation: Some(RetainedCopyActivation {
                definition: source.card,
                ability,
            }),
            ..CopyExceptions::NONE
        });
        self.apply_temporary_copy(
            source.object,
            recipe,
            CopyDeadline::WhileSource {
                source: source.object,
            },
        );
        if let Some(row) = self
            .state
            .temporary_copies
            .as_mut()
            .and_then(|rows| rows.iter_mut().find(|r| r.object == source.object))
        {
            row.layers.last_mut().unwrap().value.retained_link_origin =
                Some(temporary_copy::RetainedActivationOrigin {
                    ability_origin: item.source_ability_origin,
                });
        }
    }
}
