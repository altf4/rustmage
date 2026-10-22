//! Repeated until-end-of-turn combat damage observers own immutable trigger templates.
use super::*;

pub(super) fn owned_bytes(templates: &Option<Box<Vec<SpellView>>>) -> usize {
    templates.as_ref().map_or(0, |templates| {
        std::mem::size_of::<Vec<SpellView>>()
            + templates
                .iter()
                .map(|item| std::mem::size_of::<SpellView>() + item.owned_bytes())
                .sum::<usize>()
    })
}
impl Game {
    pub(super) fn remember_combat_draw_observer(&mut self, item: &SpellView, ability: u16) {
        let mut template = item.clone();
        template.kind = StackKind::TriggeredAbility { ability };
        template.cast = None;
        template.is_copy = false;
        template.targets.clear();
        self.state
            .delayed_combat_draws
            .get_or_insert_with(Default::default)
            .push(template);
    }
    pub(super) fn delayed_combat_draw_triggers(&mut self, events: &[DamageEvent]) {
        let Some(templates) = self.state.delayed_combat_draws.as_ref() else {
            return;
        };
        // Each creature/recipient deals damage once in a simultaneous batch. A
        // creature damaging both a planeswalker and a player triggers twice.
        let hits: std::collections::BTreeSet<_> = events
            .iter()
            .filter(|event| {
                event.combat
                    && event.amount > 0
                    && event.source.facts.battlefield
                    && event.source.facts.is_creature()
                    && match event.target {
                        Target::Player(_) => true,
                        Target::Card(id) => {
                            self.on_battlefield(id) && self.current_kinds(id) & 64 != 0
                        }
                        _ => false,
                    }
            })
            .map(|event| {
                (
                    event.source.reference,
                    event.source.controller,
                    event.target,
                )
            })
            .collect();
        let pending: Vec<_> = templates
            .iter()
            .flat_map(|template| {
                hits.iter()
                    .filter(move |(_, controller, _)| *controller == template.controller)
                    .map(move |_| template.clone())
            })
            .collect();
        for mut trigger in pending {
            trigger.id = self.next_stack_id();
            self.state.pending_triggers.push(trigger);
        }
    }
}
