//! Exact watched objects with independent, source-controller-owned death triggers.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedDeathView {
    pub watched: ObjectRef,
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
    pub ability: u16,
    pub expires_turn: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DelayedDeath {
    pub view: DelayedDeathView,
    pub item: SpellView,
}
impl Game {
    pub(super) fn delay_target_death_this_turn(&mut self, item: &SpellView, ability: u16) {
        assert_eq!(
            item.trigger_definition().triggers()[usize::from(ability)].condition,
            TriggerCondition::Reflexive,
            "instruction-owned delayed trigger program"
        );
        for bound in &item.targets {
            if !self.same_target_object(*bound) {
                continue;
            }
            let Target::Card(id) = bound.target else {
                continue;
            };
            if !self.on_battlefield(id) {
                continue;
            }
            let watched = self.object_ref(id);
            let mut captured = item.clone();
            captured.targets.clear();
            captured.mode = None;
            self.state
                .roles
                .get_or_insert_with(Default::default)
                .delayed_deaths
                .push(DelayedDeath {
                    view: DelayedDeathView {
                        watched,
                        source: item.object,
                        source_card: item.trigger_definition(),
                        controller: item.controller,
                        ability,
                        expires_turn: self.state.turn,
                    },
                    item: captured,
                });
        }
    }
    pub(super) fn delayed_death_events(&mut self, event: &ZoneChange) {
        if !event.moved || event.before.zone != Zone::Battlefield {
            return;
        }
        // Remove only the departing incarnation. No allocation or clock change
        // occurs for ordinary events without a watched recipient.
        while let Some(index) = self.state.roles.as_ref().and_then(|state| {
            state
                .delayed_deaths
                .iter()
                .position(|row| row.view.watched == event.before.reference)
        }) {
            let row = self
                .state
                .roles
                .as_mut()
                .unwrap()
                .delayed_deaths
                .remove(index);
            if event.destination != Zone::Graveyard || row.view.expires_turn != self.state.turn {
                continue;
            }
            let mut item = row.item;
            item.id = self.next_stack_id();
            item.kind = StackKind::TriggeredAbility {
                ability: row.view.ability,
            };
            // The creating spell's controller owns this delayed ability. It is
            // not an ability granted to the watched permanent (CR603.7d–e).
            self.state.pending_triggers.push(item);
        }
    }
    pub(super) fn expire_delayed_deaths(&mut self) {
        if let Some(state) = &mut self.state.roles {
            state
                .delayed_deaths
                .retain(|row| row.view.expires_turn > self.state.turn);
        }
    }
}
