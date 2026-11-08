#![allow(
    clippy::box_collection,
    reason = "Rare conspire declarations use one pointer in ordinary announcements"
)]
//! Optional conspire declarations, exact paired tap costs and independent on-cast packets.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ConspireGrant {
    pub source: ObjectRef,
    pub definition: Card,
    pub group: u16,
    pub fragment: u16,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConspireDeclaration {
    pub grant: ConspireGrant,
    pub declared: bool,
    pub pay: bool,
    pub objects: Option<[ObjectRef; 2]>,
    pub paid: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn bytes(rows: &Option<Box<Vec<ConspireDeclaration>>>) -> usize {
    rows.as_ref().map_or(0, |v| {
        std::mem::size_of::<Vec<ConspireDeclaration>>()
            + v.len() * std::mem::size_of::<ConspireDeclaration>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn conspire_grants(&self, player: Player, card: Card) -> Vec<ConspireDeclaration> {
        let mut rows = Vec::new();
        if !self.has_conspire(player, card) {
            return rows;
        }
        let evaluation = ability_evaluation::evaluate_if_needed(self);
        if card.kind_mask() & 1 != 0 {
            return rows;
        }
        for &id in &self.state.static_sources {
            let o = &self.state.objects[id as usize];
            if o.zone != Zone::Battlefield || o.controller != player {
                continue;
            }
            for (group, spec) in o.card.static_ability_groups().iter().enumerate() {
                for (fragment, f) in spec.fragments.iter().enumerate() {
                    if let StaticAbilityFragment::NoncreatureConspire = *f
                        && evaluation
                            .as_ref()
                            .is_none_or(|e| e.group_available(self.object_ref(id), group))
                    {
                        rows.push(ConspireDeclaration {
                            grant: ConspireGrant {
                                source: self.object_ref(id),
                                definition: o.card,
                                group: u16::try_from(group).expect("static group index"),
                                fragment: u16::try_from(fragment).expect("static fragment index"),
                            },
                            declared: false,
                            pay: false,
                            objects: None,
                            paid: false,
                        });
                    }
                }
            }
        }
        rows
    }
    pub(super) fn has_conspire(&self, player: Player, card: Card) -> bool {
        card.kind_mask() & 1 == 0
            && self.state.static_sources.iter().any(|&id| {
                self.state.objects[id as usize].controller == player
                    && self.active_rule_fragment(id, |f| {
                        matches!(f, StaticAbilityFragment::NoncreatureConspire)
                    })
            })
    }
    pub(super) fn conspire_declarations(
        &self,
        item: &SpellView,
    ) -> Option<Box<Vec<ConspireDeclaration>>> {
        if item.kind != StackKind::Spell || item.is_copy {
            return None;
        }
        let rows = self.conspire_grants(item.controller, item.source?.card);
        (!rows.is_empty()).then(|| Box::new(rows))
    }
    pub(super) fn choose_conspire(&mut self, pay: bool) {
        let a = self.state.announcing.as_mut().unwrap();
        let row = a
            .conspire
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|r| !r.declared)
            .unwrap();
        row.declared = true;
        row.pay = pay;
        self.continue_announcement_choices(false);
    }
    pub(super) fn queue_conspire_copies(&mut self, a: &Announcement) {
        if a.item.kind != StackKind::Spell || a.item.is_copy {
            return;
        }
        let evaluation = ability_evaluation::evaluate_if_needed(self);
        for row in a.conspire.as_deref().into_iter().flatten() {
            let grant = row.grant;
            if !row.paid
                || row.objects.is_none()
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
            trigger.kind = StackKind::ConspireCopy;
            trigger.mode = None;
            trigger.targets.clear();
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
