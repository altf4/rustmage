//! Exact-incarnation, until-cleanup replacements for battlefield-to-graveyard moves.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExileDeathReplacementView {
    pub object: ObjectRef,
    /// Historical effect source; leaving does not revoke the replacement.
    pub source: ObjectRef,
    pub source_card: Card,
    pub source_controller: Player,
    pub expires_turn: u32,
}

impl Game {
    pub(super) fn register_exile_if_dies(&mut self, item: &SpellView) {
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
            let rule = ExileDeathReplacementView {
                object: self.object_ref(id),
                source: item.object,
                source_card: item.trigger_definition(),
                source_controller: item.controller,
                expires_turn: self.state.turn,
            };
            self.state
                .death_exile_replacements
                .get_or_insert_with(|| Box::new(Vec::new()))
                .push(rule);
        }
    }

    pub(super) fn death_exile_destination(&self, before: &ObjectView, requested: Zone) -> Zone {
        if before.zone == Zone::Battlefield
            && requested == Zone::Graveyard
            && self
                .state
                .death_exile_replacements
                .as_ref()
                .is_some_and(|rules| {
                    rules.iter().any(|rule| {
                        rule.object == before.reference && rule.expires_turn == self.state.turn
                    })
                })
        {
            // Every currently admitted rule has the same mandatory destination.
            // Applying one makes all other would-die rules inapplicable; source
            // ordering cannot change the final event or leave an intermediate GY.
            Zone::Exile
        } else {
            requested
        }
    }

    pub(super) fn forget_death_exile_replacements(&mut self, object: ObjectRef) {
        if let Some(rules) = self.state.death_exile_replacements.as_mut() {
            rules.retain(|rule| rule.object != object);
            if rules.is_empty() {
                self.state.death_exile_replacements = None;
            }
        }
    }

    pub(super) fn death_exile_bytes(&self) -> usize {
        self.state
            .death_exile_replacements
            .as_ref()
            .map_or(0, |rules| {
                std::mem::size_of::<Vec<ExileDeathReplacementView>>()
                    + rules.len() * std::mem::size_of::<ExileDeathReplacementView>()
            })
    }
}
