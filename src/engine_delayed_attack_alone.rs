//! Until-turn-end observers outlive the Saga and trigger once each combat.
use super::*;
impl Game {
    pub(super) fn remember_attack_alone_observer(&mut self, item: &SpellView, ability: u16) {
        let mut template = item.clone();
        template.kind = StackKind::TriggeredAbility { ability };
        template.targets.clear();
        template.cast = None;
        template.is_copy = false;
        template.mode = None;
        self.state
            .delayed_attack_alone
            .get_or_insert_with(Default::default)
            .push(template);
    }
    pub(super) fn delayed_attack_alone_triggers(&mut self, controller: Player, attacker: CardId) {
        let templates = self
            .state
            .delayed_attack_alone
            .as_deref()
            .map_or_else(Vec::new, |rows| {
                rows.iter()
                    .filter(|row| row.controller == controller)
                    .cloned()
                    .collect()
            });
        for mut template in templates {
            template.id = self.next_stack_id();
            template.entering_object = Some(self.object_ref(attacker));
            self.state.pending_triggers.push(template);
        }
    }
}
