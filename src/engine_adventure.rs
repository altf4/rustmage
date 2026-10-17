//! One physical card, immutable chosen spell traits and exact Adventure exile rights.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AdventurePermission {
    pub object: ObjectRef,
    pub card: Card,
    pub player: Player,
}
/// Public chosen spell facts; immutable metadata is borrowed, never stored per object.
#[derive(Clone, Debug, Serialize)]
pub struct SpellCharacteristicsView {
    pub id: u64,
    pub object: ObjectRef,
    pub physical_card: Card,
    pub face: CastFace,
    pub name: &'static str,
    pub mana_cost: &'static str,
    pub mana_value: u64,
    pub type_line: &'static str,
    pub colors: u8,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct AdventureState {
    pub permissions: Vec<AdventurePermission>,
}
impl AdventureState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.permissions.len() * std::mem::size_of::<AdventurePermission>()
    }
}
impl Game {
    pub fn spell_characteristics(&self) -> Vec<SpellCharacteristicsView> {
        self.state
            .stack
            .iter()
            .chain(self.state.announcing.iter().map(|a| &a.item))
            .chain(self.state.resolving.iter().map(|r| &r.item))
            .filter(|s| s.kind == StackKind::Spell)
            .map(|s| {
                let info = s.source_card.info();
                SpellCharacteristicsView {
                    id: s.id,
                    object: s.object,
                    physical_card: s
                        .cast
                        .and_then(|c| c.physical_card)
                        .unwrap_or(s.source_card),
                    face: s.cast.map_or(CastFace::Normal, |c| c.face),
                    name: info.name,
                    mana_cost: info.mana_cost,
                    mana_value: s.mana_value(),
                    type_line: info.type_line,
                    colors: s.source_card.colors(),
                }
            })
            .collect()
    }
    pub(super) fn can_play_adventure(&self, player: Player, id: CardId) -> bool {
        self.state.adventures.as_ref().is_some_and(|rights| {
            rights.permissions.iter().any(|r| {
                r.player == player
                    && r.object.card == id
                    && r.object == self.object_ref(id)
                    && self.state.objects[id as usize].zone == Zone::Exile
            })
        })
    }
    pub(super) fn forget_adventure(&mut self, object: ObjectRef) {
        if let Some(rights) = self.state.adventures.as_mut() {
            rights.permissions.retain(|r| r.object != object);
            if rights.permissions.is_empty() {
                self.state.adventures = None;
            }
        }
    }
    /// A popped/resolving packet still owns the last stack characteristics.
    pub(super) fn move_adventure_spell(
        &mut self,
        item: &SpellView,
        destination: Zone,
    ) -> ZoneChange {
        let mut before = self.object_view(item.card);
        if before.zone == Zone::Stack && before.reference == item.object {
            before.card = item.source_card;
            before.kinds = item.source_card.kind_mask();
            before.keywords = item.source_keywords;
            before.power = 0;
            before.toughness = 0;
        }
        let event = self.transition_object(before, destination, false);
        self.finish_zone_events(std::slice::from_ref(&event), &[]);
        event
    }
    pub(super) fn finish_adventure(&mut self, item: &SpellView) {
        if self.state.objects[item.card as usize].zone != Zone::Stack
            || self.object_ref(item.card) != item.object
        {
            return;
        }
        let event = self.move_adventure_spell(item, Zone::Exile);
        if event.moved && event.destination == Zone::Exile {
            let card = self.state.objects[item.card as usize].card;
            self.state
                .adventures
                .get_or_insert_with(|| Box::new(AdventureState::default()))
                .permissions
                .push(AdventurePermission {
                    object: event.after,
                    card,
                    player: item.controller,
                });
        }
    }
    pub(super) fn finish_omen(&mut self, item: &SpellView) {
        if self.state.objects[item.card as usize].zone != Zone::Stack
            || self.object_ref(item.card) != item.object
        {
            return;
        }
        let owner = self.state.objects[item.card as usize].owner;
        let event = self.move_adventure_spell(item, Zone::Library);
        if event.moved && event.destination == Zone::Library {
            self.forget_library_top(owner);
            self.state
                .rng
                .shuffle(&mut self.state.players[owner.index()].library);
        }
    }
    /// Real stack and announcement packets carry the chosen immutable definition.
    pub(super) fn effective_spell_card(&self, id: CardId) -> Card {
        if let Some(a) = &self.state.announcing
            && a.item.kind == StackKind::Spell
            && a.item.card == id
        {
            return a.item.source_card;
        }
        self.state
            .stack
            .iter()
            .chain(self.state.resolving.iter().map(|r| &r.item))
            .find(|s| s.kind == StackKind::Spell && s.card == id)
            .map_or(self.state.objects[id as usize].card, |s| s.source_card)
    }
    pub(super) fn alternate_cast_available(
        &self,
        actor: Player,
        id: CardId,
        sorcery: bool,
        definition: Card,
    ) -> bool {
        let object = &self.state.objects[id as usize];
        // Adventure's own permission is normal-only. Another generic permission
        // remains independently usable; never deny by an "on adventure" flag.
        let generic_exile = object.zone == Zone::Exile
            && (self.can_cast_warped(actor, id)
                || self.state.exile_permissions.iter().any(|r| {
                    r.player == actor
                        && r.object == self.object_ref(id)
                        && r.expires_turn >= self.state.turn
                }));
        if object.zone != Zone::Hand && !generic_exile {
            return false;
        }
        if !sorcery
            && !definition.is_instant()
            && !definition.info().keywords.contains(Keywords::FLASH)
            && !self.cast_timing_granted(actor, definition)
        {
            return false;
        }
        let plan = match definition.effect() {
            Effect::Spell(p, _) | Effect::Aura(p) => p,
            _ => return false,
        };
        if !self.cast_targets_available(plan, actor, self.object_ref(id)) {
            return false;
        }
        if definition.x_symbols() > 0 {
            return !self
                .scalar_x_intervals_for_definition(actor, id, definition)
                .is_empty();
        }
        definition.cost_options().any(|cost| {
            let reduction = self.spell_reduction_for_definition(
                actor,
                id,
                definition.fixed_mana_value(),
                definition,
            );
            self.can_fund(FundingGoal {
                actor,
                required: cost.colored,
                generic: cost
                    .generic
                    .saturating_sub(u32::try_from(reduction).unwrap_or(u32::MAX)),
                reserve: None,
                source: None,
            })
        })
    }
}
