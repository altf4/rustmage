//! Generic-only artifact contributions, shared by actual and borrowed worlds.
use super::*;
use crate::cards::StaticAbilityFragment;
use ability_world::CharacteristicWorld;

pub(super) fn potential<W: CharacteristicWorld + ?Sized>(world: &W, definition: Card) -> bool {
    definition.has_improvise()
        || world.static_sources().iter().any(|&id| {
            world.object(id).zone == Zone::Battlefield
                && world
                    .object(id)
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|g| {
                        g.fragments
                            .iter()
                            .any(|f| matches!(f, StaticAbilityFragment::SpellImprovise(_)))
                    })
        })
}
pub(super) fn available<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    definition: Card,
) -> bool {
    definition.has_improvise() || world.static_sources().iter().any(|&id| {
        let source = world.object(id);
        source.zone == Zone::Battlefield && source.controller == player
            && source.card.static_ability_groups().iter().any(|g| g.fragments.iter()
                .any(|f| matches!(f, StaticAbilityFragment::SpellImprovise(filter) if filter.matches(definition))))
            && world.fragment_available(id, |f| matches!(f, StaticAbilityFragment::SpellImprovise(filter) if filter.matches(definition)))
    })
}
pub(super) fn eligible<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    object: ObjectRef,
) -> bool {
    if world
        .game()
        .state
        .objects
        .get(object.card as usize)
        .is_none()
    {
        return false;
    }
    let o = world.object(object.card);
    world.reference(object.card) == object
        && o.zone == Zone::Battlefield
        && o.controller == player
        && !o.tapped
        && world.kinds(object.card) & 16 != 0
}
pub(super) fn masks<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    convoke: bool,
    artifact: bool,
    excluded: &[ObjectRef],
) -> [u64; 32] {
    let mut masks = [0; 32];
    for id in 0..world.game().state.objects.len() {
        let id = id as CardId;
        let o = world.object(id);
        let r = world.reference(id);
        if o.zone != Zone::Battlefield
            || o.controller != player
            || o.tapped
            || excluded.contains(&r)
        {
            continue;
        }
        let kinds = world.kinds(id);
        let mask = if convoke && kinds & 1 != 0 {
            usize::from(world.colors(id) & 31)
        } else if artifact && kinds & 16 != 0 {
            0
        } else {
            continue;
        };
        masks[mask] += 1;
    }
    masks
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ImproviseChoiceView {
    pub spell: ObjectRef,
    pub selected: Vec<ObjectRef>,
    pub remaining: ManaCost,
    pub can_pay: bool,
}
impl Game {
    pub(super) fn item_improvise(&self, item: &SpellView) -> bool {
        item.kind == StackKind::Spell
            && item
                .source
                .is_some_and(|source| available(self, item.controller, source.card))
    }
    pub fn improvise_choice(&self) -> Option<ImproviseChoiceView> {
        if !matches!(
            self.state.pending,
            Pending::Convoke { .. } | Pending::Improvise { .. }
        ) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let f = a.convoke.as_ref()?;
        if !self.item_improvise(&a.item) && f.improvised.is_empty() {
            return None;
        }
        Some(ImproviseChoiceView {
            spell: a
                .item
                .source
                .expect("cost announcement has object source")
                .object,
            selected: f.improvised.clone(),
            remaining: f.remaining,
            can_pay: self.mana_payment_domain().is_some(),
        })
    }
    pub(super) fn choose_improviser(&mut self, object: ObjectRef) {
        let f = self
            .state
            .announcing
            .as_mut()
            .unwrap()
            .convoke
            .as_mut()
            .unwrap();
        f.remaining.generic = f
            .remaining
            .generic
            .checked_sub(1)
            .expect("generic contribution");
        f.improvised.push(object);
    }
    pub(super) fn tap_contributors_valid(&self, a: &Announcement) -> bool {
        let Some(f) = &a.convoke else {
            return true;
        };
        f.selected
            .iter()
            .all(|s| self.convoke_eligible(a.item.controller, s.object))
            && (f.improvised.is_empty()
                || self.item_waterbend(&a.item) != 0
                    && f.improvised
                        .iter()
                        .all(|&r| waterbend::eligible(self, a.item.controller, r))
                || self.item_improvise(&a.item)
                    && f.improvised
                        .iter()
                        .all(|&r| eligible(self, a.item.controller, r)))
    }
}
