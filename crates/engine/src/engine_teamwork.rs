//! Optional teamwork declarations and simultaneous, non-targeting creature-tap costs.
use super::*;
use crate::cards::StaticAbilityFragment as F;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TeamworkChoiceView {
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub selected_power: Option<i128>,
    pub can_finish: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn teamwork_mode_allowed(&self, item: &SpellView, mode: usize) -> bool {
        let Some(spec) = item.source.and_then(|source| source.card.teamwork()) else {
            return true;
        };
        let bits = if item.cast.is_some_and(|c| c.teamwork == Some(true)) {
            spec.paid_modes
        } else {
            spec.normal_modes
        };
        mode < 64 && bits & (1u64 << mode) != 0
    }
    pub(super) fn teamwork_intent_fundable(&self, announcement: &Announcement, pay: bool) -> bool {
        announcement
            .item
            .source
            .and_then(|source| source.card.teamwork())
            .is_some_and(|spec| {
                !pay || self.optional_creature_material_available(announcement, Some(spec.minimum))
            })
    }

    /// Blight needs one controlled creature of any power. Teamwork needs the
    /// requested total power among untapped creatures, including summoning-sick ones.
    pub(super) fn optional_creature_material_available(
        &self,
        announcement: &Announcement,
        minimum_power: Option<u16>,
    ) -> bool {
        let state = &self.state;
        if !state.proposal_affordability
            || !payment_capacity::stable_optional_material_board(self, announcement)
        {
            return true;
        }
        let usable = |object: &Object| {
            object.zone == Zone::Battlefield
                && object.controller == announcement.item.controller
                && (minimum_power.is_none() || !object.tapped)
        };
        let affected = |reference: ObjectRef| {
            let object = &state.objects[reference.card as usize];
            object.generation == reference.generation && usable(object)
        };
        // Keep uncertain local forms. A persistent type-setting form can
        // exclude the creature type if every relevant animation agrees.
        let mut noncreature_forms = Vec::new();
        if let Some(roles) = &state.roles {
            for row in roles.animations.iter().filter(|row| affected(row.object)) {
                if row.expires_turn.is_some()
                    || row.while_source_tapped.is_some()
                    || row.creature_only
                    || !(row.enchantment_noncreature || row.artifact_noncreature || row.food_only)
                {
                    return true;
                }
                if noncreature_forms.is_empty() {
                    noncreature_forms.resize(state.objects.len(), false);
                }
                noncreature_forms[row.object.card as usize] = true;
            }
        }
        if state.roles.as_ref().is_some_and(|roles| {
            !roles.characteristic_sources.is_empty() || !roles.melded_permanents.is_empty()
        }) || state
            .crew
            .as_ref()
            .is_some_and(|crew| crew.animated.iter().copied().any(affected))
            || state
                .copy_recipes
                .as_ref()
                .is_some_and(|rows| rows.iter().any(|row| affected(row.object)))
        {
            return true;
        }
        let simple_power = minimum_power.is_some()
            && state
                .objects
                .iter()
                .filter(|object| matches!(object.zone, Zone::Battlefield | Zone::Command))
                .all(|object| {
                    object
                        .card
                        .static_ability_groups()
                        .iter()
                        .flat_map(|group| group.fragments)
                        .all(|fragment| {
                            matches!(
                                fragment,
                                F::IntrinsicKeywords(_)
                                    | F::CannotBeBlockedByCreaturesWithLessPower
                                    | F::ChosenBasicLandType
                                    | F::CostReduction(..)
                                    | F::PlotCostReduction(..)
                                    | F::StartYourEngines
                                    | F::StaticCastRestriction(..)
                                    | F::HarnessedTrigger(_)
                                    | F::AdditionalLandPlays(_)
                                    | F::PlayLandFromGraveyard
                                    | F::EntryTriggerSuppression(..)
                            )
                        })
                })
            && state.attachment_sources.is_empty()
            && state.self_granted_abilities.is_none()
            && state.roles.as_ref().is_none_or(|roles| {
                roles.attached.is_empty()
                    && roles.base_stats.is_empty()
                    && roles.until_next_turn_stats.is_empty()
                    && roles.live_static.is_empty()
            })
            && state.entry_values.as_ref().is_none_or(|rows| {
                rows.iter().all(|row| {
                    let object = &state.objects[row.object.card as usize];
                    !matches!(object.zone, Zone::Battlefield | Zone::Command)
                        || object.generation != row.object.generation
                        || !object.card.static_ability_groups().iter().any(|group| {
                            group.fragments.iter().any(|fragment| {
                                matches!(
                                    fragment,
                                    crate::cards::StaticAbilityFragment::ChosenModifier(_)
                                )
                            })
                        })
                })
            });
        let mut needed = i128::from(minimum_power.unwrap_or(0));
        for (id, object) in state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, object)| usable(object))
        {
            let card = object.card;
            if card
                .static_ability_groups()
                .iter()
                .any(|group| group.first_layer <= crate::cards::AbilityLayer::Type)
            {
                return true;
            }
            if !card.is_creature() || noncreature_forms.get(id) == Some(&true) {
                continue;
            }
            if minimum_power.is_none()
                || !simple_power
                || card.dynamic_stats().is_some()
                || card.vehicle_stats().is_some()
            {
                return true;
            }
            let Effect::Creature(base, _) = card.effect() else {
                return true;
            };
            // Negative power and -1/-1 counters can only lower the achievable sum.
            let upper = i128::from(base)
                .max(0)
                .saturating_add(object.growth.power.max(0))
                .saturating_add(i128::from(object.counters.plus_one));
            needed = needed.saturating_sub(upper);
            if needed <= 0 {
                return true;
            }
        }
        minimum_power == Some(0)
    }
    pub(super) fn choose_teamwork(&mut self, pay: bool) {
        let improvise = improvise::potential(
            self,
            self.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .source
                .expect("cast announcement has source")
                .card,
        );
        let a = self.state.announcing.as_mut().unwrap();
        a.item.cast.as_mut().unwrap().teamwork = Some(pay);
        a.initialize_cast_costs(improvise);
        let item = a.item.clone();
        let plan = self.target_spec(&item);
        self.state.announcing.as_mut().unwrap().plan = plan;
        self.continue_announcement_choices(false);
    }
    pub(super) fn visit_teamwork_payments(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let selected = &a.cast_costs.as_ref().unwrap().teammates;
        for (id, o) in self.state.objects.iter().enumerate() {
            let object = self.object_ref(id as CardId);
            if o.zone != Zone::Battlefield
                || o.controller != a.item.controller
                || o.tapped
                || !self.is_creature(id as CardId)
                || selected.contains(&object)
            {
                continue;
            }
            let mut next = selected.clone();
            next.push(object);
            if self.teamwork_prefix_fundable(&next, false)
                && emit(Action::ChooseTeammate { object })
            {
                return true;
            }
        }
        self.teamwork_prefix_fundable(selected, true) && emit(Action::FinishTeamwork)
    }
    pub(super) fn choose_teammate(&mut self, object: ObjectRef) {
        self.state
            .announcing
            .as_mut()
            .unwrap()
            .cast_costs
            .as_mut()
            .unwrap()
            .teammates
            .push(object);
    }
    pub(super) fn finish_teamwork(&mut self) {
        let a = self.state.announcing.as_mut().unwrap();
        let c = a.cast_costs.as_mut().unwrap();
        let active = c.active.take().unwrap();
        assert!(matches!(
            c.clauses[usize::from(active)].clause,
            CastCostClause::Teamwork(_)
        ));
        c.clauses[usize::from(active)].paid = true;
        let ids = c.teammates.iter().map(|r| r.card).collect::<Vec<_>>();
        let actor = a.item.controller;
        self.tap_objects_as(actor, &ids);
        // This event is distinct from merely becoming tapped. Queue after the
        // complete simultaneous cost; these triggers wait until casting finishes.
        for id in ids {
            let source = self.object_view(id);
            for (index, ability) in source.trigger_templates() {
                if matches!(
                    ability.condition.event(),
                    TriggerCondition::SelfPaysTeamwork
                ) {
                    self.queue_trigger(&source, index);
                }
            }
        }
        self.finish_announcement();
    }
    pub fn teamwork_choice(&self) -> Option<TeamworkChoiceView> {
        if !matches!(self.state.pending, Pending::TeamworkPayment { .. }) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let selected = &a.cast_costs.as_ref()?.teammates;
        let mut sum = saddle::PowerSum::default();
        for r in selected {
            sum.add(self.stats(r.card).power);
        }
        Some(TeamworkChoiceView {
            minimum: a
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .teamwork()?
                .minimum,
            selected: selected.clone(),
            selected_power: sum.represented(),
            can_finish: self.teamwork_prefix_fundable(selected, true),
        })
    }
}
