//! One physical card, immutable chosen spell traits and exact Adventure exile rights.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AdventurePermission {
    pub object: ObjectRef,
    pub card: Card,
    pub player: Player,
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
    pub(super) fn finish_adventure(&mut self, item: &SpellView) {
        if self.state.objects[item.card as usize].zone != Zone::Stack
            || self.object_ref(item.card) != item.object
        {
            return;
        }
        let event = self.move_object(item.card, Zone::Exile);
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
    pub(super) fn adventure_cast_available(
        &self,
        actor: Player,
        id: CardId,
        sorcery: bool,
    ) -> bool {
        let object = &self.state.objects[id as usize];
        let Some(definition) = object.card.adventure_spell() else {
            return false;
        };
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
        if !sorcery && !definition.is_instant() {
            return false;
        }
        let plan = match definition.effect() {
            Effect::Spell(p, _) | Effect::Aura(p) => p,
            _ => return false,
        };
        self.cast_targets_available(plan, actor, self.object_ref(id))
            && definition.cost_options().any(|cost| {
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
