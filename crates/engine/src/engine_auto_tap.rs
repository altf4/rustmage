//! Optional caller policy for one locked spell or activation. The action menu never invokes it.
use super::ability_world::CharacteristicWorld;
use super::auto_tap_solver::{self, Mode, Pools, Source, SpendBonus};
use super::*;
use crate::cards::CastFilter;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
#[path = "engine_auto_tap_followup.rs"]
mod followup;
pub use followup::AutoTapFollowUp;

#[cfg(feature = "test-support")]
thread_local! {
    static INVENTORY_COUNTS: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

/// Caller preferences apply to this request only. Source identities include their incarnation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AutoTapOptions {
    pub protected_sources: Vec<ObjectRef>,
    pub follow_up: Option<ManaCost>,
    /// An observed source and normal spell or printed battlefield activation to hold available.
    pub follow_up_action: Option<AutoTapFollowUp>,
    pub allow_treasure: bool,
    pub allow_life_payment: bool,
    /// Prefer spending up to this many distinct colors on this spell or ability.
    pub prefer_distinct_colors: u8,
}

impl Default for AutoTapOptions {
    fn default() -> Self {
        Self {
            protected_sources: Vec::new(),
            follow_up: None,
            follow_up_action: None,
            allow_treasure: true,
            allow_life_payment: false,
            prefer_distinct_colors: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoTapStatus {
    Completed,
    Paid,
    NeedsDecision,
    InsufficientSupportedMana,
}

/// A successful preview and its execution at the same revision have the same primitive trace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AutoTapResult {
    pub status: AutoTapStatus,
    pub attempt_id: u64,
    /// Revision at the start of planning, before any returned primitive.
    pub revision: u64,
    pub actions: Vec<Action>,
    pub payment: Option<Mana>,
}

impl AutoTapResult {
    fn new(attempt_id: u64, revision: u64) -> Self {
        Self {
            status: AutoTapStatus::NeedsDecision,
            attempt_id,
            revision,
            actions: Vec::new(),
            payment: None,
        }
    }
}

/// Only immutable costs and public resource data enter the assignment algorithm.
/// Keep the action for each mode in the same order as its solver input.
#[derive(Default)]
struct TapInputs {
    sources: Vec<Source>,
    actions: Vec<Vec<Action>>,
    followups: Vec<ManaCost>,
}

fn deterministic_output(recipe: ManaProduction) -> bool {
    matches!(
        recipe.base(),
        ManaProduction::Fixed(_)
            | ManaProduction::Choice { .. }
            | ManaProduction::ChosenColor
            | ManaProduction::ChosenColorAmount(_)
            | ManaProduction::CountSubtype { .. }
    )
}

impl Game {
    #[cfg(feature = "test-support")]
    pub fn take_auto_tap_inventory_counts() -> (usize, usize) {
        INVENTORY_COUNTS.with(|counts| counts.replace((0, 0)))
    }

    /// Most continuous effects never change a mana ability or its cost. Remove those
    /// providers from the index consulted during pricing; the real game still validates
    /// each selected action. Only cheap, unconditional source guards are priced against
    /// this narrower index. A dense, interacting layer state is left to manual payment.
    fn auto_tap_inventory_world(&self) -> Option<Cow<'_, Game>> {
        use crate::cards::StaticAbilityFragment as F;
        const MAX_INTERACTING_PROVIDERS: usize = 8;
        let state = &self.state;
        // Some of the engine's exact queries evaluate a whole layer or scan an unbounded
        // sparse registry. Avoid building one such evaluation for each ready resource.
        if state.roles.as_ref().is_some_and(|r| {
            !r.resolved_ability_modifications.is_empty()
                || !r.ability_removal_sources.is_empty()
                || !r.source_linked_effects.is_empty()
                || r.characteristic_sources.len() > MAX_INTERACTING_PROVIDERS
                || r.animations.len() > MAX_INTERACTING_PROVIDERS
                || r.basic_land_types.len() > MAX_INTERACTING_PROVIDERS
                || r.creature_types.len() > MAX_INTERACTING_PROVIDERS
                || r.keyword_effects.len() > MAX_INTERACTING_PROVIDERS
                || r.harnessed.len() > MAX_INTERACTING_PROVIDERS
        }) || state.attachment_sources.len() > MAX_INTERACTING_PROVIDERS
            || state
                .self_granted_abilities
                .as_ref()
                .is_some_and(|rows| rows.len() > MAX_INTERACTING_PROVIDERS)
            || state
                .entry_values
                .as_ref()
                .is_some_and(|rows| rows.len() > MAX_INTERACTING_PROVIDERS)
            || state
                .manifest
                .as_ref()
                .is_some_and(|m| m.permanents.len() > MAX_INTERACTING_PROVIDERS)
        {
            return None;
        }
        let mut interacting = Vec::new();
        for &id in &state.static_sources {
            let object = &state.objects[id as usize];
            if !matches!(object.zone, Zone::Battlefield | Zone::Command)
                || self.is_face_down(self.object_ref(id))
            {
                continue;
            }
            let groups = object.card.static_ability_groups();
            let price_neutral = !groups.is_empty()
                && groups.iter().all(|group| {
                    group.fragments.iter().all(|fragment| {
                        matches!(
                            fragment,
                            F::DeathBaronLord
                                | F::StaticModifier(..)
                                | F::ScalingModifier(..)
                                | F::IntrinsicKeywords(_)
                                | F::ControlledCreatureScaling { .. }
                                | F::ControlledLegendaryCreatureBonus(_)
                                | F::ControlledCreatureBonusPerSubtype { .. }
                                | F::CreatureGroupBonus { .. }
                                | F::CreatureGroupScaling { .. }
                        )
                    })
                });
            if !price_neutral {
                interacting.push(id);
                if interacting.len() > MAX_INTERACTING_PROVIDERS {
                    return None;
                }
            }
        }
        let world = if interacting.len() == state.static_sources.len() {
            Cow::Borrowed(self)
        } else {
            let mut world = self.clone();
            world.state.static_sources = interacting;
            Cow::Owned(world)
        };
        (!ability_evaluation::needs_ability_evaluation(world.as_ref())).then_some(world)
    }

    /// Positive values indicate a recognized benefit, negative values an immediate
    /// loss from its compulsory draw or life payment. Use the public number of cards
    /// in the library, including every applicable live observer.
    fn auto_tap_treasure_preference(
        &self,
        a: &Announcement,
        reserved_life: i128,
        known_printed: bool,
    ) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if a.item.kind != StackKind::Spell {
            return Ordering::Equal;
        }
        let payer = a.item.controller;
        let face_down: HashSet<_> = self
            .state
            .manifest
            .iter()
            .flat_map(|state| &state.permanents)
            .map(|row| row.object)
            .collect();
        let count = self
            .state
            .cast_sources
            .iter()
            .filter(|&&id| {
                let object = &self.state.objects[id as usize];
                object.zone == Zone::Battlefield
                    && object.controller == payer
                    && !face_down.contains(&self.object_ref(id))
                    && object.card.triggers().iter().any(|t| {
                        matches!(
                            t.condition,
                            TriggerCondition::TreasureManaSpent(&TriggerCondition::SpellCast {
                                relation: CastRelation::You,
                                filter: CastFilter::Any,
                                ordinal: None
                            })
                        ) && matches!(
                            t.program,
                            [
                                Instruction::Draw(PlayerSet::Controller, 1),
                                Instruction::LoseLife(PlayerSet::Controller, 1)
                            ]
                        )
                    })
            })
            .count();
        if count == 0 {
            Ordering::Equal
        } else if self.state.players[payer.index()].life <= reserved_life + count as i128
            || self.state.players[payer.index()].library.len() < count
        {
            Ordering::Less
        } else if known_printed {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }

    fn auto_tap_inputs(
        &self,
        a: &Announcement,
        purpose: mana_lots::Purpose,
        options: &AutoTapOptions,
        avoid_treasure: bool,
        reserved_life: i128,
    ) -> TapInputs {
        let payer = a.item.controller;
        let mut inputs = TapInputs::default();
        let mut concealed_draws = HashSet::new();
        let mut cursor = self.state.cast_rollback.as_deref();
        while let Some(attempt) = cursor {
            if !attempt.costs_complete {
                concealed_draws.extend(attempt.deferred_draws.iter().map(|draw| draw.object));
            }
            cursor = attempt.parent.as_deref();
        }
        let mut reserved: HashSet<_> = options.protected_sources.iter().copied().collect();
        a.visit_declared_cost_objects(|object| {
            reserved.insert(object);
        });
        reserved.extend(a.item.attached_activation_tap_cost());
        if let Some(source) = a.item.source
            && let StackKind::ActivatedAbility { ability } = a.item.kind
            && source.card.activation(ability).cost.tap
        {
            reserved.insert(source.object);
        }
        let open = !self.attempt_mana_window_closed();
        let mut subtype_yields = HashMap::new();
        for (index, object) in self.state.objects.iter().enumerate() {
            let id = index as CardId;
            if object.zone == Zone::Hand
                && object.owner == payer
                && !concealed_draws.contains(&self.object_ref(id))
                && !object.card.is_land()
                && a.item
                    .source
                    .is_none_or(|source| source.object != self.object_ref(id))
            {
                let (colored, generic) = object.card.cost();
                inputs.followups.push(ManaCost { colored, generic });
            }
            if object.zone != Zone::Battlefield
                || object.controller != payer
                || self.is_face_down(self.object_ref(id))
            {
                continue;
            }
            if !open || object.tapped || reserved.contains(&self.object_ref(id)) {
                continue;
            }
            #[cfg(feature = "test-support")]
            INVENTORY_COUNTS.with(|counts| {
                let (ready, interacting) = counts.get();
                counts.set((
                    ready + 1,
                    interacting
                        + usize::from(
                            !self.state.static_sources.is_empty()
                                || !self.state.attachment_sources.is_empty(),
                        ),
                ));
            });
            let creature = self.is_creature(id);
            if creature
                && object.summoning_sick
                && !self.activation_keywords(id).contains(Keywords::HASTE)
                || self.named_activation_generic_increase(id) != 0
                || self.activation_prohibited(payer, id)
            {
                continue;
            }
            let treasure = self.subtypes(id).has(object.card, "Treasure");
            if treasure && avoid_treasure {
                continue;
            }
            let expendable = treasure && object.is_token() && !creature && options.allow_treasure;
            let mut source = Source {
                modes: Vec::new(),
                creature,
                variable: false,
                treasure,
                followup_eligible: true,
            };
            let mut actions = Vec::new();
            self.visit_intrinsic_mana(id, payer, &mut |action| {
                if matches!(action, Action::TapMana { .. }) {
                    let mut mana = [0; 6];
                    mana[self.intrinsic_mana_color(id).unwrap()] = 1;
                    source.modes.push(Mode {
                        mana,
                        pain: false,
                        sacrifice: false,
                    });
                    actions.push(action);
                }
                false
            });
            // These grants include additional basic land types on duals and triomes.
            for grant in self.granted_mana(id) {
                if !deterministic_output(grant.production)
                    || grant.sacrifice && (!expendable || self.sacrifice_prohibited(id))
                {
                    continue;
                }
                if let Some(quality) = self.mana_quality(grant.production, id) {
                    if !quality.eligible(purpose) {
                        continue;
                    }
                    source.followup_eligible &=
                        quality.restriction == ManaRestrictionView::Unrestricted;
                }
                // A source with a free tap is reusable. Keep its free ability in preference to
                // a granted self-sacrifice with an otherwise identical output.
                grant
                    .production
                    .visit_at_start_with_fixed_choice(self, id, false, |mana| {
                        if mana != [0; 6] {
                            source.modes.push(Mode {
                                mana,
                                pain: false,
                                sacrifice: grant.sacrifice,
                            });
                            actions.push(Action::TapGrantedMana {
                                card: id,
                                grantor: grant.source,
                                production: mana,
                            });
                        }
                        false
                    });
            }
            for (index, ability) in object.card.activations().iter().enumerate() {
                let index = index as u16;
                let cost = ability.cost;
                if ability.origin != ActivationOrigin::Battlefield
                    || ability.guard != crate::cards::ActivationGuard::Always
                    || object.card.self_granted_activation_template(index)
                    || object.card.grants_activation(index)
                {
                    continue;
                }
                let [Instruction::AddMana(recipe)] = ability.program else {
                    // Keep ordinary visible utility activations in the follow-up cost sketches.
                    if !ability.sorcery
                        && ability.guard == crate::cards::ActivationGuard::Always
                        && !ability.has_use_limit()
                        && !ability.has_x_cost()
                        && !cost.sacrifice
                        && !cost.exile_self
                        && !cost.discard
                        && cost.additional.is_none()
                        && cost.life == 0
                        && cost.remove.is_none()
                        && cost.loyalty.is_none()
                        && (cost.generic != 0 || cost.colored != [0; 6])
                        && !cost.tap
                    {
                        inputs.followups.push(ManaCost {
                            colored: cost.colored,
                            generic: cost.generic,
                        });
                    }
                    continue;
                };
                if object.card.legacy_subtype_mana_activation(index)
                    || !deterministic_output(*recipe)
                    || !cost.tap
                    || cost.colored != [0; 6]
                    || cost.generic != 0
                    || !cost.alternatives.is_empty()
                    || cost.additional.is_some()
                    || cost.remove.is_some()
                    || cost.loyalty.is_some()
                    || cost.discard
                    || cost.exile_self
                    || ability.has_x_cost()
                    || object.card.activation_put_counter(index).is_some()
                    || (cost.life != 0
                        && (!options.allow_life_payment
                            || self.state.players[payer.index()].life
                                <= reserved_life + i128::from(cost.life)))
                    || cost.sacrifice && !expendable
                    || !self.activation_cost_available(
                        id,
                        ability,
                        usize::from(index),
                        payer,
                        false,
                    )
                {
                    continue;
                }
                if let Some(quality) = self.mana_quality(*recipe, id) {
                    if !quality.eligible(purpose) {
                        continue;
                    }
                    source.followup_eligible &=
                        quality.restriction == ManaRestrictionView::Unrestricted;
                }
                let domain = match recipe.base() {
                    ManaProduction::CountSubtype { color, subtype } => {
                        source.variable = true;
                        *subtype_yields
                            .entry((color, subtype))
                            .or_insert_with(|| recipe.in_world(self, id))
                    }
                    _ => recipe.in_world(self, id),
                };
                domain.visit(|mana| {
                    let pain = cost.life != 0;
                    if mana != [0; 6] {
                        source.modes.push(Mode {
                            mana,
                            pain,
                            sacrifice: cost.sacrifice,
                        });
                        actions.push(Action::ActivateMana {
                            card: id,
                            ability: index,
                            payment: [0; 6],
                            counter_payment: Counters::default(),
                            production: mana,
                        });
                    }
                    false
                });
            }
            if !source.modes.is_empty() {
                inputs.sources.push(source);
                inputs.actions.push(actions);
            }
        }
        inputs
    }
}
impl Game {
    /// Preview a payment without changing the live game or its revision.
    pub fn plan_mana_payment(
        &self,
        actor: Player,
        revision: u64,
        attempt_id: u64,
        options: &AutoTapOptions,
    ) -> Result<AutoTapResult, Error> {
        self.auto_tap_branch(actor, revision, attempt_id, options)
            .map(|(result, _)| result)
    }

    /// Commit exactly one validated payment, retaining the normal per-action revisions and receipts.
    pub fn auto_pay_mana(
        &mut self,
        actor: Player,
        revision: u64,
        attempt_id: u64,
        options: &AutoTapOptions,
    ) -> Result<AutoTapResult, Error> {
        let (result, branch) = self.auto_tap_branch(actor, revision, attempt_id, options)?;
        if let Some(branch) = branch {
            *self = branch;
        }
        Ok(result)
    }

    fn auto_tap_branch(
        &self,
        actor: Player,
        revision: u64,
        attempt_id: u64,
        options: &AutoTapOptions,
    ) -> Result<(AutoTapResult, Option<Game>), Error> {
        // Source queries can encounter the same recoverable numeric limits as a raw action.
        // Planning never owns the live state; expose any such failure through the ordinary API.
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.build_auto_tap_branch(actor, revision, attempt_id, options)
        })) {
            Ok(result) => result,
            Err(payload) => match payload.downcast::<CapacitySignal>() {
                Ok(signal) => Err(Error::CapacityExceeded(signal.0)),
                Err(payload) => std::panic::resume_unwind(payload),
            },
        }
    }

    fn build_auto_tap_branch(
        &self,
        actor: Player,
        revision: u64,
        attempt_id: u64,
        options: &AutoTapOptions,
    ) -> Result<(AutoTapResult, Option<Game>), Error> {
        if self.revision() != revision {
            return Err(Error::StaleRevision);
        }
        if self.actor() != Some(actor) {
            return Err(if self.state.outcome.is_some() {
                Error::GameOver
            } else {
                Error::WrongActor
            });
        }
        let Some(attempt) = self.state.cast_rollback.as_ref().filter(|a| {
            a.attempt_id == attempt_id
                && attempt_id != 0
                && self.shares_information(actor, a.player)
        }) else {
            return Err(Error::StaleRevision);
        };
        if options.prefer_distinct_colors > 5 {
            return Err(Error::IllegalAction);
        }
        let followup = self.auto_tap_followup(attempt.player, options)?;
        let mut result = AutoTapResult::new(attempt_id, revision);
        let Some(a) = self.state.announcing.as_ref() else {
            return Ok((result, None));
        };
        // A parent can be suspended during an entry, a replacement or another player's choice.
        // A pay request does not authorize making declarations or committing a contributor selection.
        if !matches!(self.state.pending, Pending::PayCosts { player } if player == attempt.player)
            || a.item.controller != attempt.player
            || Some(a.item.id) != attempt.context
            || a.total_cost.is_none()
            || a.production.is_some()
            || self.state.entry_batch.is_some()
            || self.state.replacements.is_some()
            || self.mana_tap_bonus_pending()
        {
            return Ok((result, None));
        }
        let Some(request) = self.announcement_payment_request(actor) else {
            return Ok((result, None));
        };
        if a.cast_costs.as_ref().is_some_and(|costs| {
            costs.mana_paid
                || costs.mana_closed && costs.active_clause() != Some(CastCostClause::Mana)
        }) {
            return Ok((result, None));
        }
        let cost = ManaCost {
            colored: request.required,
            generic: request.generic,
        };
        let purpose = self
            .purpose_for_item(&a.item)
            .with_live_characteristics(self, a.item.controller);
        let mut pool = self.available_mana(a.item.controller, purpose);
        let reserved_life = a.unpaid_auto_tap_life();
        let inventory = self.auto_tap_inventory_world();
        let treasure_preference =
            self.auto_tap_treasure_preference(a, reserved_life, inventory.is_some());
        let avoid_treasure = treasure_preference.is_lt();
        let mut treasure_pool = [0u32; 6];
        let mut followup_pool = pool;
        for lot in self.mana_lots(a.item.controller) {
            if lot.quality.eligible(purpose) {
                for (i, amount) in lot.amounts.into_iter().enumerate() {
                    let amount = u32::try_from(amount).expect("subset of representable pool");
                    if lot.quality.treasure_source && avoid_treasure {
                        pool[i] -= amount;
                    } else if lot.quality.treasure_source {
                        treasure_pool[i] += amount;
                    }
                    if lot.quality.restriction != ManaRestrictionView::Unrestricted
                        || lot.quality.treasure_source && avoid_treasure
                    {
                        followup_pool[i] -= amount;
                    }
                }
            }
        }
        let mut inputs = inventory.as_ref().map_or_else(TapInputs::default, |world| {
            world.auto_tap_inputs(a, purpose, options, avoid_treasure, reserved_life)
        });
        let bonus = SpendBonus {
            distinct_colors: if options.prefer_distinct_colors == 0
                && a.item.kind == StackKind::Spell
                && a.item.source.is_some_and(|source| {
                    source.card.entry_counter_rules().iter().any(|rule| {
                        matches!(
                            rule.amount,
                            EntryCounterAmount::ColorsSpent
                                | EntryCounterAmount::ColorsSpentKind(CounterKind::PlusOne)
                        )
                    })
                }) {
                5
            } else {
                options.prefer_distinct_colors
            },
            treasure: treasure_preference.is_gt(),
        };
        let Some(solution) = inputs.solve_with_followup(
            cost,
            Pools {
                eligible: pool,
                treasure: treasure_pool,
                followup: followup_pool,
            },
            followup,
            bonus,
        ) else {
            if inventory.is_some() {
                result.status = AutoTapStatus::InsufficientSupportedMana;
            }
            return Ok((result, None));
        };
        // One branch executes the selected actions. A rejected payment or an unexpected
        // decision drops that branch, including its provisional taps and mana-window state.
        let mut branch = self.clone();
        let mut actions = Vec::with_capacity(solution.taps.len() + 1);
        for (source, mode) in solution.taps {
            let action = inputs.actions[source][mode].clone();
            if inputs.sources[source].modes[mode].pain
                && let Action::ActivateMana { card, ability, .. } = &action
                && branch.state.players[attempt.player.index()].life
                    <= reserved_life
                        + i128::from(
                            branch.state.objects[*card as usize]
                                .card
                                .activation(*ability)
                                .cost
                                .life,
                        )
            {
                return Ok((result, None));
            }
            match branch.apply(actor, action.clone()) {
                Ok(()) => actions.push(action),
                Err(Error::IllegalAction | Error::WrongActor | Error::GameOver) => {
                    return Ok((result, None));
                }
                Err(error) => return Err(error),
            }
            if !matches!(branch.state.pending, Pending::PayCosts { player } if player == attempt.player)
                || branch.state.announcing.as_ref().map(|a| a.item.id) != attempt.context
                || branch.state.entry_batch.is_some()
                || branch.state.replacements.is_some()
                || branch.mana_tap_bonus_pending()
            {
                return Ok((result, None));
            }
        }
        let pay = Action::PayCosts {
            payment: solution.payment,
        };
        if !avoid_treasure
            && solution.prefer_treasure
            && branch
                .auto_tap_treasure_preference(
                    branch.state.announcing.as_ref().unwrap(),
                    reserved_life,
                    false,
                )
                .is_lt()
        {
            return Ok((result, None));
        }
        match branch.apply(actor, pay.clone()) {
            Ok(()) => actions.push(pay.clone()),
            Err(Error::IllegalAction | Error::WrongActor | Error::GameOver) => {
                return Ok((result, None));
            }
            Err(error) => return Err(error),
        }
        let lot_debits = branch.auto_tap_lot_debits(treasure_preference);
        while let Some(range) = branch.mana_lot_spend_range() {
            if range.action != pay || branch.actor() != Some(actor) {
                return Ok((result, None));
            }
            let desired = lot_debits[range.lot][range.color].clamp(range.minimum, range.maximum);
            let amount = range.values.as_ref().map_or(desired, |values| {
                let next = values.partition_point(|v| *v < desired);
                values.get(next).copied().unwrap_or(range.maximum)
            });
            let action = Action::ChooseManaLotSpend { amount };
            branch.apply(actor, action.clone())?;
            actions.push(action);
        }
        let completed =
            branch.state.last_attempt_outcome[attempt.player.index()].is_some_and(|outcome| {
                outcome.attempt_id == attempt_id && outcome.outcome == AttemptOutcome::Completed
            });
        let paid = branch.state.announcing.as_ref().is_some_and(|a| {
            Some(a.item.id) == attempt.context
                && a.cast_costs.as_ref().is_some_and(|costs| costs.mana_paid)
        });
        if !completed && !paid {
            return Ok((result, None));
        }
        result.status = if completed {
            AutoTapStatus::Completed
        } else {
            AutoTapStatus::Paid
        };
        result.actions = actions;
        result.payment = Some(solution.payment);
        Ok((result, Some(branch)))
    }
}

impl Announcement {
    /// Life already committed by the enclosing action is unavailable for optional source costs.
    fn unpaid_auto_tap_life(&self) -> i128 {
        let mut life = match (self.item.kind, self.item.source) {
            (StackKind::ActivatedAbility { ability }, Some(source)) => {
                i128::from(source.card.activation(ability).cost.life)
            }
            _ => 0,
        };
        let mut additional_paid = false;
        if let Some(costs) = &self.cast_costs {
            for row in &costs.clauses {
                match row.clause {
                    CastCostClause::Life(amount) | CastCostClause::AlternativeLife(amount)
                        if !row.paid =>
                    {
                        life += i128::from(amount);
                    }
                    CastCostClause::Additional => additional_paid = row.paid,
                    _ => {}
                }
            }
        }
        if !additional_paid
            && let Some(spec) = self.additional_cost()
            && (self.additional_life || matches!(spec.kind, EffectChoiceKind::PayLife(_)))
        {
            life += i128::from(spec.kind.additional_life_payment().unwrap_or(0));
        }
        life
    }
}
