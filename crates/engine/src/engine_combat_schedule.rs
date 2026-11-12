//! Inserted phases and source-independent repeated combat packets.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MainPhaseCombatView {
    /// Inserted combats not yet begun; the current combat is already consumed.
    pub remaining: u64,
    /// Successor of the main phase captured when the effect resolved.
    pub resume: Step,
}

/// A run of identical phases at the next combat boundary, newest first in observations.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AfterCombatPhaseView {
    pub first_step: Step,
    pub remaining: u128,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub land_creature_attackers_only: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn begin_combat_rules_turn(&mut self) {
        let Some(mut rules) = self.state.combat_rules.take() else {
            return;
        };
        let active = self.state.active;
        let mut continuing = rules.require_all_attack_until_next_turn;
        continuing[active.index()] = 0;
        let turn = self.state.turn;
        rules.turn_callbacks.retain_mut(|row| {
            if row.expiry != Some(TurnCallbackExpiry::EndOfNextTurn) {
                return false;
            }
            if row.item.controller == active {
                row.expiry = Some(TurnCallbackExpiry::EndOfTurn(turn));
            }
            true
        });
        if continuing != [0; 2] || !rules.turn_callbacks.is_empty() {
            let turn_callbacks = std::mem::take(&mut rules.turn_callbacks);
            *rules = Default::default();
            rules.require_all_attack_until_next_turn = continuing;
            rules.turn_callbacks = turn_callbacks;
            self.state.combat_rules = Some(rules);
        }
    }

    pub(super) fn require_all_attacks_until_next_turn(&mut self, controller: Player) {
        let requirements = &mut self
            .state
            .combat_rules
            .get_or_insert_with(Default::default)
            .require_all_attack_until_next_turn[controller.index()];
        *requirements = requirements
            .checked_add(1)
            .capacity("global attack requirements exhausted");
    }

    pub(super) fn global_attack_requirements(&self) -> [usize; 2] {
        self.state
            .combat_rules
            .as_ref()
            .map_or([0; 2], |rules| rules.require_all_attack_until_next_turn)
    }

    pub(super) fn prohibit_attacks_on_controller_or_planeswalkers_this_turn(
        &mut self,
        item: &SpellView,
        players: PlayerSet,
    ) {
        for player in self.program_players(players, item) {
            self.state
                .combat_rules
                .get_or_insert_with(Default::default)
                .player_attack_prohibitions[player.index()] |= 1 << item.controller.index();
        }
    }

    pub(super) fn require_created_attacks_this_combat(&mut self, created: &[ObjectRef]) {
        if !matches!(
            self.state.step,
            Step::BeginCombat
                | Step::Attackers
                | Step::Blockers
                | Step::FirstStrikeDamage
                | Step::CombatDamage
                | Step::EndCombat
        ) {
            return;
        }
        let actual: Vec<_> = created
            .iter()
            .copied()
            .filter(|r| self.on_battlefield(r.card) && self.object_ref(r.card) == *r)
            .collect();
        if actual.is_empty() {
            return;
        }
        let rows = &mut self
            .state
            .combat_rules
            .get_or_insert_with(Default::default)
            .must_attack_this_combat;
        for object in actual {
            rows.retain(|old| old.card != object.card || *old == object);
            if !rows.contains(&object) {
                rows.push(object);
            }
        }
    }
    pub(super) fn add_combats_after_current_main(&mut self, count: u16) {
        if count == 0 {
            return;
        }
        let resume = match self.state.step {
            Step::Main1 => Step::BeginCombat,
            Step::Main2 => Step::End,
            _ => return,
        };
        let rules = self.state.combat_rules.get_or_insert_with(Default::default);
        if !rules.inserted_phases.is_empty() {
            rules
                .inserted_phases
                .extend(std::iter::repeat_n(Step::BeginCombat, usize::from(count)));
            return;
        }
        let schedule = rules.after_main.get_or_insert(MainPhaseCombatView {
            remaining: 0,
            resume,
        });
        assert_eq!(
            schedule.resume, resume,
            "one current main-phase continuation"
        );
        schedule.remaining = schedule
            .remaining
            .checked_add(u64::from(count))
            .capacity("main-phase combat count exhausted");
    }
    /// Each pair is inserted after this main, before its older successor (CR500.8).
    pub(super) fn add_combat_main_pairs(&mut self, count: u16) {
        if count == 0 || !matches!(self.state.step, Step::Main1 | Step::Main2) {
            return;
        }
        let natural = if self.state.step == Step::Main1 {
            Step::BeginCombat
        } else {
            Step::End
        };
        let rules = self.state.combat_rules.get_or_insert_with(Default::default);
        if rules.inserted_phases.is_empty() {
            if let Some(old) = rules.after_main.take() {
                rules.inserted_phases.push(old.resume);
                for _ in 0..old.remaining {
                    rules.inserted_phases.push(Step::BeginCombat);
                }
            } else {
                rules.inserted_phases.push(natural);
            }
        }
        for _ in 0..count {
            rules
                .inserted_phases
                .extend([Step::Main2, Step::BeginCombat]);
        }
    }
    pub(super) fn step_after_main(&mut self) -> Step {
        if let Some(step) = self
            .state
            .combat_rules
            .as_mut()
            .and_then(|rules| rules.inserted_phases.pop())
        {
            return step;
        }
        if self.take_main_phase_combat() || self.state.step == Step::Main1 {
            Step::BeginCombat
        } else {
            Step::End
        }
    }
    pub(super) fn take_main_phase_combat(&mut self) -> bool {
        if let Some(schedule) = self
            .state
            .combat_rules
            .as_mut()
            .and_then(|rules| rules.after_main.as_mut())
            && schedule.remaining != 0
        {
            schedule.remaining -= 1;
            true
        } else {
            false
        }
    }
    pub(super) fn add_combat(&mut self, land_creature_attackers_only: bool) {
        self.insert_after_combat(Step::BeginCombat, 1, land_creature_attackers_only);
    }

    pub(super) fn add_upkeeps_after_combat(&mut self, controller: Player, amount: u128) {
        // CR 500.10a: "you get" cannot add phases to another player's turn.
        if amount != 0 && self.attacking_return_combat(controller) {
            self.insert_after_combat(Step::Upkeep, amount, false);
        }
    }

    fn insert_after_combat(
        &mut self,
        first_step: Step,
        count: u128,
        land_creature_attackers_only: bool,
    ) {
        let rules = self.state.combat_rules.get_or_insert_with(Default::default);
        let phases = &mut rules.after_combat;
        // CR 500.8: the latest added phase occurs first at the same boundary.
        if let Some(last) = phases.last_mut().filter(|last| {
            last.first_step == first_step
                && last.land_creature_attackers_only == land_creature_attackers_only
        }) {
            last.remaining = last
                .remaining
                .checked_add(count)
                .capacity("additional phase count exhausted");
        } else {
            phases
                .try_reserve(1)
                .capacity("additional phase allocation exhausted");
            phases.push(AfterCombatPhaseView {
                first_step,
                remaining: count,
                land_creature_attackers_only,
            });
        }
    }

    fn take_after_combat_phase(&mut self) -> Option<Step> {
        let rules = self.state.combat_rules.as_mut()?;
        rules.land_creature_attackers_only = false;
        let phase = rules.after_combat.last_mut()?;
        let step = phase.first_step;
        rules.land_creature_attackers_only = phase.land_creature_attackers_only;
        phase.remaining -= 1;
        if phase.remaining == 0 {
            rules.after_combat.pop();
        }
        rules.in_inserted_upkeep = step == Step::Upkeep;
        Some(step)
    }

    pub(super) fn step_after_inserted_upkeep(&mut self) -> Option<Step> {
        let rules = self.state.combat_rules.as_mut()?;
        if !std::mem::take(&mut rules.in_inserted_upkeep) {
            return None;
        }
        Some(self.step_after_combat())
    }

    pub(super) fn step_after_combat(&mut self) -> Step {
        if let Some(step) = self.take_after_combat_phase() {
            return step;
        }
        if let Some(step) = self
            .state
            .combat_rules
            .as_mut()
            .and_then(|rules| rules.inserted_phases.pop())
        {
            return step;
        }
        if self.take_main_phase_combat() {
            return Step::BeginCombat;
        }
        self.state
            .combat_rules
            .as_mut()
            .and_then(|rules| rules.after_main.take())
            .map_or(Step::Main2, |schedule| schedule.resume)
    }
    pub(super) fn schedule_begin_combat_this_turn(&mut self, item: &SpellView, ability: u16) {
        let mut packet = item.clone();
        packet.id = self.next_stack_id();
        packet.kind = StackKind::TriggeredAbility { ability };
        packet.targets.clear();
        packet.clear_initiating_event_references();
        packet.target_type_slots = None;
        packet.mode = None;
        packet.cast = None;
        packet.is_copy = false;
        packet.related_stack = None;
        packet.reflexive_subject = None;
        packet.additional_triggers = 0;
        self.state
            .combat_rules
            .get_or_insert_with(Default::default)
            .begin_combat_this_turn
            .get_or_insert_default()
            .push(packet);
    }
    pub(super) fn queue_begin_combat_this_turn(&mut self) {
        let packets: Vec<_> = self
            .state
            .combat_rules
            .as_ref()
            .and_then(|rules| rules.begin_combat_this_turn.as_deref())
            .into_iter()
            .flatten()
            .cloned()
            .collect();
        for mut packet in packets {
            packet.id = self.next_stack_id();
            self.state.pending_triggers.push(packet);
        }
    }
    pub(super) fn untap_creatures_that_attacked_this_turn(&mut self) {
        // Resolve the current set after responses; control does not qualify it.
        // Tokens put onto the battlefield attacking never enter this history.
        let ids: Vec<_> = self
            .state
            .attacked_this_turn
            .iter()
            .copied()
            .filter(|&object| {
                self.on_battlefield(object.card)
                    && self.object_ref(object.card) == object
                    && self.is_creature(object.card)
            })
            .map(|object| object.card)
            .collect();
        self.untap_objects(&ids, untap_rules::UntapCause::Effect);
    }
}
