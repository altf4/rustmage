//! Resolution-only replacements retain an exact actual cast independently of its source.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DreamExileView {
    pub object: ObjectRef,
    pub player: Player,
    pub face: CastFace,
    pub resolving: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn mark_cast_dream_exile(&mut self, item: &SpellView) {
        let Some(event) = &item.cast_event else {
            return;
        };
        let Some(spell) = self.state.stack.iter().find(|s| {
            s.kind == StackKind::Spell
                && !s.is_copy
                && s.source.is_some_and(|source| source.object == event.object)
        }) else {
            return;
        };
        let mark = DreamExileView {
            object: spell.source.unwrap().object,
            player: spell.controller,
            face: spell.cast.map_or(CastFace::Normal, |c| c.face),
            resolving: false,
        };
        let rows = &mut self.state.zone_rules.get_or_insert_default().dream_exiles;
        if !rows.iter().any(|r| r.object == mark.object) {
            rows.push(mark);
        }
    }
    pub(super) fn dream_resolution_mark(&self, object: ObjectRef) -> Option<DreamExileView> {
        self.state
            .zone_rules
            .as_ref()?
            .dream_exiles
            .iter()
            .find(|r| r.object == object && r.resolving)
            .copied()
    }
    pub(super) fn forget_dream_exile(&mut self, object: ObjectRef) {
        if let Some(rules) = &mut self.state.zone_rules {
            rules.dream_exiles.retain(|r| r.object != object);
        }
    }
    pub(super) fn finish_dream_spell_resolution(&mut self, item: &SpellView) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        let Some(mark) = self.state.zone_rules.as_mut().and_then(|s| {
            s.dream_exiles
                .iter_mut()
                .find(|r| r.object == source.object)
        }) else {
            return false;
        };
        mark.resolving = true;
        mark.player = item.controller;
        mark.face = item.cast.map_or(CastFace::Normal, |c| c.face);
        self.move_adventure_spell(item, Zone::Graveyard);
        true
    }
    pub(super) fn start_dream_free_cast(&mut self, player: Player) {
        let remaining = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.owner == player
                    && o.zone == Zone::Exile
                    && o.is_card()
                    && self.counter_count(*id as CardId, CounterKind::Dream) > 0
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect();
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            uncast_to_hand: false,
            siege_transformed: false,
            paid: None,
            casts_remaining: Some(1),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player,
            remaining,
        }));
        self.state.pending = Pending::ResolutionCasts { player };
    }
    pub(super) fn complete_special_zone_move(
        &mut self,
        object: ObjectRef,
        completion: zone_replacements::ZoneMoveCompletion,
        was_card: bool,
    ) -> Option<(Player, u8)> {
        match completion {
            zone_replacements::ZoneMoveCompletion::Dream => {
                self.add_counter(object.card, CounterKind::Dream, 1)
            }
            zone_replacements::ZoneMoveCompletion::Adventure(player) => {
                let card = self.state.objects[object.card as usize].card;
                self.state
                    .adventures
                    .get_or_insert_default()
                    .permissions
                    .push(AdventurePermission {
                        object,
                        card,
                        player,
                    });
            }
            zone_replacements::ZoneMoveCompletion::LinkedDeathExile(replacement) => {
                if was_card {
                    self.link_exile_to_ability(
                        replacement.source,
                        replacement.ability_origin,
                        object,
                    );
                }
                return Some((replacement.controller, replacement.life));
            }
        }
        None
    }
}
