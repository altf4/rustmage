//! Non-targeted battlefield sets, frozen before simultaneous movement or damage.
use super::*;
impl Game {
    fn battlefield_filter_matches(
        &self,
        filter: BattlefieldFilter,
        actor: Player,
        id: CardId,
    ) -> bool {
        let object = &self.state.objects[id as usize];
        object.zone == Zone::Battlefield
            && (filter.kinds == 0 || self.current_kinds(id) & filter.kinds != 0)
            && (filter.relation == -1 || (object.controller == actor) == (filter.relation == 0))
            && (!filter.nonland || !object.card.is_land())
            && filter.tapped.is_none_or(|tapped| object.tapped == tapped)
            && (filter.without == Keywords::NONE || !self.keywords(id).intersects(filter.without))
            && filter
                .subtype
                .is_none_or(|subtype| self.has_subtype(id, subtype))
            && (filter.max_mana == u32::MAX
                || object.card.fixed_mana_value() <= u64::from(filter.max_mana))
    }
    fn battlefield_group(&self, item: &SpellView, filters: &[BattlefieldFilter]) -> Vec<CardId> {
        (0..self.state.objects.len())
            .filter_map(|id| {
                let id = id as CardId;
                filters
                    .iter()
                    .any(|&filter| self.battlefield_filter_matches(filter, item.controller, id))
                    .then_some(id)
            })
            .collect()
    }
    pub(super) fn move_battlefield_group(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        exile: bool,
    ) {
        let mut ids = self.battlefield_group(item, filters);
        if !exile {
            ids.retain(|&id| !self.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        }
        let sources = if exile || ids.is_empty() {
            Vec::new()
        } else {
            self.death_trigger_sources()
        };
        let events = self.move_objects_raw(&ids, if exile { Zone::Exile } else { Zone::Graveyard });
        self.state.resolving.as_mut().unwrap().moved_count = Some(
            u64::try_from(events.iter().filter(|e| e.moved).count())
                .capacity("group result count exhausted"),
        );
        // Returns caused by the frozen departure event may suspend for entry
        // choices. The result count survives that pause before later instructions.
        self.finish_zone_events(&events, &sources);
    }
    pub(super) fn return_battlefield_group(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        attacking: bool,
        players: Option<PlayerSet>,
    ) {
        // Resolve the player target first, but do not target any group member:
        // hexproof, ward and shroud on those permanents do not apply.
        let controllers = players.map(|set| {
            let mut selected = [false; 2];
            for player in self.program_players(set, item) {
                selected[player.index()] = true;
            }
            selected
        });
        let ids: Vec<_> = self
            .battlefield_group(item, filters)
            .into_iter()
            .filter(|&id| {
                (!attacking || self.state.attackers.iter().any(|a| a.card == id))
                    && controllers.is_none_or(|selected| {
                        selected[self.state.objects[id as usize].controller.index()]
                    })
            })
            .collect();
        // One frozen group preserves simultaneous departure/LKI. The existing
        // zone mover returns stolen objects to their owners and handles tokens.
        let events = self.move_objects_raw(&ids, Zone::Hand);
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn program_damage_source(&self, item: &SpellView) -> DamageSource {
        if self.object_ref(item.card) == item.object {
            self.damage_source(item.card)
        } else {
            DamageSource {
                facts: item.source_facts,
                object: item.card,
                reference: item.object,
                controller: item.controller,
                keywords: item.source_keywords,
            }
        }
    }
    pub(super) fn damage_battlefield_group(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        players: Option<PlayerSet>,
        amount: i32,
    ) {
        let source = self.program_damage_source(item);
        let mut recipients: Vec<_> = self
            .battlefield_group(item, filters)
            .into_iter()
            .map(Target::Card)
            .collect();
        if let Some(players) = players {
            recipients.extend(self.program_players(players, item).map(Target::Player));
        }
        let events: Vec<_> = recipients
            .into_iter()
            .map(|target| DamageEvent {
                source,
                target,
                amount: i128::from(amount),
                combat: false,
            })
            .collect();
        self.apply_damage_batch(&events);
    }
}
