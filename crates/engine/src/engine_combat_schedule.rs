//! Main-relative extra combat phases and source-independent repeated combat packets.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MainPhaseCombatView {
    /// Inserted combats not yet begun; the current combat is already consumed.
    pub remaining: u64,
    /// Successor of the main phase captured when the effect resolved.
    pub resume: Step,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
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
    pub(super) fn step_after_combat(&mut self) -> Step {
        // An ability creating a combat during the current combat inserts it
        // before the previously scheduled successors, without losing their return.
        if self.take_additional_combat() || self.take_main_phase_combat() {
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
        packet.crime_qualifying_target = false;
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
