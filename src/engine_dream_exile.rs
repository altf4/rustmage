//! Resolution-only dream exile replacements and a source-independent owned-card cast window.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DreamExileView {
    pub object: ObjectRef,
    pub player: Player,
    pub face: CastFace,
    pub resolving: bool,
}
impl Game {
    pub(super) fn mark_cast_dream_exile(&mut self, item: &SpellView) {
        let Some(event) = item.cast_event.as_ref() else {
            return;
        };
        let Some(spell) = self
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell && !s.is_copy && s.object == event.object)
        else {
            return;
        };
        let mark = DreamExileView {
            object: spell.object,
            player: spell.controller,
            face: spell.cast.map_or(CastFace::Normal, |c| c.face),
            resolving: false,
        };
        // A later resolving action may suspend for a competing replacement.
        // Register checkpoint ownership before that action begins.
        self.state.zone_rules.get_or_insert_default();
        let rows = &mut self.state.roles.get_or_insert_default().dream_exiles;
        if !rows.iter().any(|r| r.object == mark.object) {
            rows.push(mark);
        }
    }
    pub(super) fn dream_resolution_mark(&self, object: ObjectRef) -> Option<DreamExileView> {
        self.state
            .roles
            .as_ref()?
            .dream_exiles
            .iter()
            .find(|r| r.object == object && r.resolving)
            .copied()
    }
    pub(super) fn forget_dream_exile(&mut self, object: ObjectRef) {
        if let Some(roles) = &mut self.state.roles {
            roles.dream_exiles.retain(|r| r.object != object);
        }
    }
    pub(super) fn finish_dream_spell_resolution(&mut self, item: &SpellView) -> bool {
        let Some(mark) = self.state.roles.as_mut().and_then(|roles| {
            roles
                .dream_exiles
                .iter_mut()
                .find(|r| r.object == item.object)
        }) else {
            return false;
        };
        mark.resolving = true;
        mark.player = item.controller;
        mark.face = item.cast.map_or(CastFace::Normal, |c| c.face);
        self.move_adventure_spell(item, Zone::Graveyard);
        true
    }
    pub(super) fn commit_dream_zone_result(
        &mut self,
        object: ObjectRef,
        effect: Option<ZoneReplacementKind>,
        mark: Option<DreamExileView>,
    ) {
        let Some(mark) = mark else {
            return;
        };
        match effect {
            Some(ZoneReplacementKind::DreamExile) => {
                self.add_counter(object.card, CounterKind::Dream, 1)
            }
            Some(ZoneReplacementKind::AdventureExile) => {
                let card = self.state.objects[object.card as usize].card;
                self.state
                    .adventures
                    .get_or_insert_default()
                    .permissions
                    .push(AdventurePermission {
                        object,
                        card,
                        player: mark.player,
                    });
            }
            Some(ZoneReplacementKind::OmenShuffle) => {
                let owner = self.state.objects[object.card as usize].owner;
                self.forget_library_top(owner);
                self.state
                    .rng
                    .shuffle(&mut self.state.players[owner.index()].library);
            }
            _ => {}
        }
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
                    && !o.is_token()
                    && self.counter_count(*id as CardId, CounterKind::Dream) > 0
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect();
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            player,
            remaining,
            maximum_mana_value: None,
            uncast_to_hand: false,
            remaining_casts: Some(1),
        }));
        self.state.pending = Pending::ResolutionCasts { player };
    }
}
