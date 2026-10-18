//! Live casting-origin and mana-type permissions, shared by real and funding worlds.
use super::*;
use crate::cards::StaticAbilityFragment;
impl Card {
    pub(super) fn has_cast_origin_abilities(self) -> bool {
        self.static_ability_groups().iter().any(|g| {
            g.fragments.iter().any(|f| {
                matches!(
                    f,
                    StaticAbilityFragment::LookAtLibraryTop
                        | StaticAbilityFragment::CastCreatureFromLibraryTop
                        | StaticAbilityFragment::SpendAnyManaForCreatures
                )
            })
        })
    }
}
fn has_fragment<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    predicate: impl Fn(&StaticAbilityFragment) -> bool,
) -> bool {
    world.game().state.static_sources.iter().any(|&id| {
        let object = world.object(id);
        object.zone == Zone::Battlefield
            && object.controller == player
            && object
                .card
                .static_ability_groups()
                .iter()
                .any(|g| g.fragments.iter().any(&predicate))
            && world.printed_abilities_available(id)
    })
}
pub(super) fn payment_requirements<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    purpose: mana_lots::Purpose,
    required: Mana,
    generic: u32,
) -> (Mana, u32) {
    if matches!(purpose, mana_lots::Purpose::Spell(source) if source.kinds & 1 != 0)
        && has_fragment(world, player, |f| {
            matches!(f, StaticAbilityFragment::SpendAnyManaForCreatures)
        })
    {
        (
            [0; 6],
            required
                .into_iter()
                .try_fold(generic, u32::checked_add)
                .capacity("mana substitution cost exhausted"),
        )
    } else {
        (required, generic)
    }
}
pub(super) fn payment_goal<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    goal: &FundingGoal,
    purpose: mana_lots::Purpose,
) -> FundingGoal {
    let (required, generic) =
        payment_requirements(world, goal.actor, purpose, goal.required, goal.generic);
    FundingGoal {
        required,
        generic,
        ..goal.clone()
    }
}
impl Game {
    pub(super) fn can_cast_library_top(&self, player: Player, card: CardId) -> bool {
        let object = &self.state.objects[card as usize];
        object.owner == player
            && object.zone == Zone::Library
            && object.card.is_creature()
            && self.state.players[player.index()].library.last() == Some(&card)
            && has_fragment(self, player, |f| {
                matches!(f, StaticAbilityFragment::CastCreatureFromLibraryTop)
            })
    }
    pub(super) fn visible_library_top(&self, viewer: Player, owner: Player) -> Option<ObjectRef> {
        let next_top_hidden = self.state.announcing.as_ref().is_some_and(|a| {
            a.item.cast.is_some_and(|c| c.origin == Zone::Library) && a.item.controller == owner
        });
        if viewer == owner
            && !next_top_hidden
            && has_fragment(self, viewer, |f| {
                matches!(f, StaticAbilityFragment::LookAtLibraryTop)
            })
        {
            self.state.players[owner.index()]
                .library
                .last()
                .map(|&id| self.object_ref(id))
        } else {
            self.public_library_top(owner)
        }
    }
}
