#![allow(
    clippy::box_collection,
    reason = "Rare casualty declarations use one pointer in ordinary announcements"
)]
//! Optional casualty declarations, exact sacrificed costs and independent on-cast packets.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CasualtyGrant {
    pub source: ObjectRef,
    pub definition: Card,
    pub group: u16,
    pub fragment: u16,
    pub minimum: u16,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CasualtyDeclaration {
    pub grant: CasualtyGrant,
    pub declared: bool,
    pub pay: bool,
    pub sacrifice: Option<ObjectRef>,
    pub paid: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn bytes(rows: &Option<Box<Vec<CasualtyDeclaration>>>) -> usize {
    rows.as_ref().map_or(0, |v| {
        std::mem::size_of::<Vec<CasualtyDeclaration>>()
            + v.len() * std::mem::size_of::<CasualtyDeclaration>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn casualty_grants(&self, player: Player, card: Card) -> Vec<CasualtyDeclaration> {
        let mut rows = Vec::new();
        if !self.has_casualty(player, card) {
            return rows;
        }
        let evaluation = ability_evaluation::evaluate_if_needed(self);
        if card.kind_mask() & 12 == 0 {
            return rows;
        }
        for &id in &self.state.static_sources {
            let o = &self.state.objects[id as usize];
            if o.zone != Zone::Battlefield || o.controller != player {
                continue;
            }
            for (group, spec) in o.card.static_ability_groups().iter().enumerate() {
                for (fragment, f) in spec.fragments.iter().enumerate() {
                    if let StaticAbilityFragment::ControlledSpellsCasualty { kinds, minimum } = *f
                        && card.kind_mask() & kinds != 0
                        && evaluation
                            .as_ref()
                            .is_none_or(|e| e.group_available(self.object_ref(id), group))
                    {
                        rows.push(CasualtyDeclaration {
                            grant: CasualtyGrant {
                                source: self.object_ref(id),
                                definition: o.card,
                                group: u16::try_from(group).expect("static group index"),
                                fragment: u16::try_from(fragment).expect("static fragment index"),
                                minimum,
                            },
                            declared: false,
                            pay: false,
                            sacrifice: None,
                            paid: false,
                        });
                    }
                }
            }
        }
        rows
    }
    pub(super) fn has_casualty(&self, player: Player, card: Card) -> bool {
        card.kind_mask() & 12 != 0 && self.state.static_sources.iter().any(|&id| {
            self.state.objects[id as usize].controller == player && self.active_rule_fragment(id, |f| matches!(f, StaticAbilityFragment::ControlledSpellsCasualty {kinds,..} if card.kind_mask() & kinds != 0))
        })
    }
    pub(super) fn casualty_declarations(
        &self,
        item: &SpellView,
    ) -> Option<Box<Vec<CasualtyDeclaration>>> {
        if item.kind != StackKind::Spell || item.is_copy {
            return None;
        }
        let rows = self.casualty_grants(item.controller, item.source?.card);
        (!rows.is_empty()).then(|| Box::new(rows))
    }
    pub(super) fn choose_casualty(&mut self, pay: bool) {
        let a = self.state.announcing.as_mut().unwrap();
        let row = a
            .casualty
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|r| !r.declared)
            .unwrap();
        row.declared = true;
        row.pay = pay;
        self.continue_announcement_choices(false);
    }
    pub(super) fn queue_casualty_copies(&mut self, a: &Announcement) {
        if a.item.kind != StackKind::Spell || a.item.is_copy {
            return;
        }
        let evaluation = ability_evaluation::evaluate_if_needed(self);
        for row in a.casualty.as_deref().into_iter().flatten() {
            let grant = row.grant;
            if !row.paid
                || row.sacrifice.is_none()
                || self.object_ref(grant.source.card) != grant.source
            {
                continue;
            }
            let source = &self.state.objects[grant.source.card as usize];
            let Some(_fragment) = source
                .card
                .static_ability_groups()
                .get(usize::from(grant.group))
                .and_then(|g| g.fragments.get(usize::from(grant.fragment)))
            else {
                continue;
            };
            if source.card != grant.definition
                || source.controller != a.item.controller
                || !self.on_battlefield(grant.source.card)
                || !evaluation
                    .as_ref()
                    .is_none_or(|e| e.group_available(grant.source, usize::from(grant.group)))
            {
                continue;
            }
            let mut trigger = a.item.clone();
            trigger.id = self.next_stack_id();
            trigger.kind = StackKind::CasualtyCopy;
            trigger.mode = None;
            trigger.targets.clear();
            trigger.crime_qualifying_target = false;
            trigger.target_type_slots = None;
            trigger.cast = None;
            trigger.is_copy = false;
            trigger.cast_event = None;
            trigger.mana_copy_sources = None;
            trigger.copy_event = Some(Box::new(CopyEvent {
                spell: a.item.clone(),
                amount: 1,
            }));
            self.state.pending_triggers.push(trigger);
        }
    }
}
