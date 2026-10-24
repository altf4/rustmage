//! Card copies created outside the stack, then cast through ordinary declarations.
use super::*;

impl Game {
    /// CR707.12: the copy is neither a token nor an already-copied spell.
    /// Normal announcements of this public object count as casting spells.
    pub(super) fn allocate_card_copy(&mut self, definition: Card, controller: Player) -> ObjectRef {
        assert!(crate::cards::CastFilter::InstantSorcery.matches(definition));
        let (id, generation) = self.state.free_tokens.pop().map_or_else(
            || {
                (
                    CardId::try_from(self.state.objects.len()).capacity("object slots exhausted"),
                    0,
                )
            },
            |id| (id, self.state.objects[id as usize].generation),
        );
        let object = Object {
            card: definition,
            copy_kind: CopyKind::Card,
            owner: controller,
            controller,
            zone: Zone::Exile,
            generation,
            tapped: false,
            summoning_sick: false,
            flashback: false,
            damage: 0,
            damaged_this_turn: false,
            dealt_damage_this_turn: false,
            deathtouch_since_sba: false,
            growth: Stats::default(),
            gained_keywords: Keywords::NONE,
            counters: InitialCounters::default(),
            used_abilities: 0,
            attached_to: None,
        };
        if id as usize == self.state.objects.len() {
            self.state
                .objects
                .try_reserve(1)
                .capacity("card copy allocation exhausted");
            self.state.objects.push(object);
        } else {
            self.state.objects[id as usize] = object;
        }
        self.object_ref(id)
    }
    /// Ceasing is not a zone change. A successfully cast copy remains on the stack.
    pub(super) fn cease_card_copy(&mut self, reference: ObjectRef) {
        let Some(object) = self.state.objects.get(reference.card as usize) else {
            return;
        };
        if object.generation != reference.generation
            || object.copy_kind != CopyKind::Card
            || matches!(object.zone, Zone::Stack | Zone::Outside)
        {
            return;
        }
        if object.zone == Zone::Library {
            self.state.players[object.owner.index()]
                .library
                .retain(|&id| id != reference.card);
        }
        self.state
            .exile_permissions
            .retain(|p| p.object != reference);
        self.state
            .exile_links
            .retain(|link| link.exiled != reference);
        self.state.known_hand.retain(|r| *r != reference);
        self.clear_copy_recipe(reference);
        self.set_counters(reference.card, Counters::default());
        let object = &mut self.state.objects[reference.card as usize];
        object.zone = Zone::Outside;
        object.generation = object
            .generation
            .checked_add(1)
            .capacity("object generation exhausted");
        self.state.free_tokens.push(reference.card);
    }
}
