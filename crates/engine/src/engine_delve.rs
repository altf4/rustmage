//! Delve is an alternative payment for locked generic mana, never mana production
//! or a cost reduction. It shares the ordinary mana contributor transaction.
use super::*;
use crate::cards::StaticAbilityFragment;
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn potential<W: CharacteristicWorld + ?Sized>(world: &W) -> bool {
    world.static_sources().iter().any(|&id| {
        world
            .object(id)
            .card
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .any(|f| matches!(f, StaticAbilityFragment::SpellDelve(_)))
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn available<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    card: Card,
) -> bool {
    world.static_sources().iter().any(|&id| {
        let source = world.object(id);
        source.zone == Zone::Battlefield && source.controller == player
            && source.card.static_ability_groups().iter().flat_map(|group| group.fragments)
                .any(|f| matches!(f, StaticAbilityFragment::SpellDelve(filter) if filter.matches(card)))
            && world.fragment_available(id, |f| matches!(f,
                StaticAbilityFragment::SpellDelve(filter) if filter.matches(card)))
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn eligible<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    object: ObjectRef,
    source: Option<ObjectRef>,
) -> bool {
    if world
        .game()
        .state
        .objects
        .get(object.card as usize)
        .is_none()
        || source == Some(object)
    {
        return false;
    }
    let card = world.object(object.card);
    world.reference(object.card) == object
        && card.zone == Zone::Graveyard
        && card.owner == player
        && card.is_card()
        && !world.graveyard_exile_prohibited(object.card)
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn count<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    source: Option<ObjectRef>,
    excluded: &[ObjectRef],
) -> u64 {
    (0..world.game().state.objects.len())
        .filter(|&id| {
            let object = world.reference(id as CardId);
            !excluded.contains(&object) && eligible(world, player, object, source)
        })
        .count() as u64
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct DelveChoiceView {
    pub spell: ObjectRef,
    pub selected: Vec<ObjectRef>,
    pub remaining: ManaCost,
    pub can_pay: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn item_delve(&self, item: &SpellView) -> bool {
        item.kind == StackKind::Spell
            && item
                .source
                .is_some_and(|source| available(self, item.controller, source.card))
    }
    pub fn delve_choice(&self) -> Option<DelveChoiceView> {
        if !matches!(
            self.state.pending,
            Pending::Convoke { .. } | Pending::Improvise { .. } | Pending::Delve { .. }
        ) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let frame = a.convoke.as_ref()?;
        if !self.item_delve(&a.item) && frame.delved.is_empty() {
            return None;
        }
        Some(DelveChoiceView {
            spell: a.item.source?.object,
            selected: frame.delved.clone(),
            remaining: frame.remaining,
            can_pay: self.mana_payment_domain().is_some(),
        })
    }
    pub(super) fn choose_delver(&mut self, object: ObjectRef) {
        let frame = self
            .state
            .announcing
            .as_mut()
            .unwrap()
            .convoke
            .as_mut()
            .unwrap();
        frame.remaining.generic = frame
            .remaining
            .generic
            .checked_sub(1)
            .expect("generic delve share");
        frame.delved.push(object);
    }
    pub(super) fn delver_fundable(&self, object: ObjectRef) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let mut world = self.cast_cost_world(a);
        world.project_delver(object) && world.cost_completion()
    }
    pub(super) fn pay_mana_contributors(&mut self, a: &mut Announcement) {
        let Some(frame) = &mut a.convoke else {
            return;
        };
        let taps = frame
            .objects()
            .iter()
            .map(|object| object.card)
            .collect::<Vec<_>>();
        self.tap_objects_as(a.item.controller, &taps);
        let delved = std::mem::take(&mut frame.delved);
        if !delved.is_empty() {
            assert!(
                delved.iter().all(|&object| eligible(
                    self,
                    a.item.controller,
                    object,
                    a.item.source.map(|s| s.object)
                )),
                "selected delve payment became illegal"
            );
            let ids = delved.iter().map(|object| object.card).collect::<Vec<_>>();
            let events = self.move_objects_raw(&ids, Zone::Exile);
            self.finish_zone_events(&events, &[]);
        }
    }
}
