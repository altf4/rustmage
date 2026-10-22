//! Optional conspire declarations and real, ordered nonmana cost payments (CR702.78).
use super::*;
use crate::cards::StaticAbilityFragment;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConspireInstanceView {
    pub source: ObjectRef,
    pub ability_origin: u64,
    pub pay: Option<bool>,
    pub paid: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConspireCastView {
    pub stack_id: u64,
    pub instances: Vec<ConspireInstanceView>,
    pub costs_started: bool,
    #[serde(skip)]
    pub(super) projecting: bool,
}
impl Game {
    pub(super) fn conspire_view(&self) -> Option<&ConspireCastView> {
        self.state.roles.as_ref()?.conspire.as_deref()
    }
    fn conspire_mut(&mut self) -> &mut ConspireCastView {
        self.state
            .roles
            .as_mut()
            .unwrap()
            .conspire
            .as_deref_mut()
            .unwrap()
    }
    pub(super) fn conspire_projecting(&self) -> bool {
        self.conspire_view().is_some_and(|c| c.projecting)
    }
    pub(super) fn conspire_has_payment(&self) -> bool {
        self.conspire_view()
            .is_some_and(|c| c.instances.iter().any(|i| i.pay == Some(true)))
    }
    pub(super) fn conspire_needs_intent(&self) -> bool {
        self.conspire_view()
            .is_some_and(|c| c.instances.iter().any(|i| i.pay.is_none()))
    }
    fn conspire_provider(&self, source: ObjectRef, player: Player, origin: u64) -> bool {
        self.on_battlefield(source.card)
            && self.object_ref(source.card) == source
            && self.state.objects[source.card as usize].controller == player
            && self.copy_ability_origin(source) == origin
            && self.state.objects[source.card as usize]
                .card
                .static_ability_groups()
                .iter()
                .any(|g| {
                    g.fragments
                        .iter()
                        .any(|f| matches!(f, StaticAbilityFragment::NoncreatureConspire))
                })
            && self.continuous_fragment_available(source.card, |f| {
                matches!(f, StaticAbilityFragment::NoncreatureConspire)
            })
    }
    pub(super) fn capture_conspire(&mut self, item: &SpellView) {
        if item.kind != StackKind::Spell || item.is_copy {
            return;
        }
        let instances = if item.source_card.is_creature() {
            Vec::new()
        } else {
            self.state
                .static_sources
                .iter()
                .copied()
                .filter_map(|id| {
                    let source = self.object_ref(id);
                    let ability_origin = self.copy_ability_origin(source);
                    self.conspire_provider(source, item.controller, ability_origin)
                        .then_some(ConspireInstanceView {
                            source,
                            ability_origin,
                            pay: None,
                            paid: false,
                        })
                })
                .collect::<Vec<_>>()
        };
        if let Some(roles) = &mut self.state.roles {
            roles.conspire = None;
        }
        if !instances.is_empty() {
            self.state.roles.get_or_insert_default().conspire = Some(Box::new(ConspireCastView {
                stack_id: item.id,
                instances,
                costs_started: false,
                projecting: false,
            }));
        }
    }
    pub(super) fn choose_conspire(&mut self, source: ObjectRef, pay: bool) {
        self.conspire_mut()
            .instances
            .iter_mut()
            .find(|i| i.source == source && i.pay.is_none())
            .unwrap()
            .pay = Some(pay);
        self.continue_announcement_choices(false);
    }
    fn conspire_pair_valid(&self, first: ObjectRef, second: ObjectRef) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        first != second
            && [first, second].into_iter().all(|r| {
                self.on_battlefield(r.card)
                    && self.object_ref(r.card) == r
                    && self.state.objects[r.card as usize].controller == a.item.controller
                    && !self.state.objects[r.card as usize].tapped
                    && self.is_creature(r.card)
                    && ability_world::CharacteristicWorld::colors(self, r.card)
                        & a.item.source_card.colors()
                        != 0
            })
    }
    pub(super) fn pay_conspire(&mut self, source: ObjectRef, first: ObjectRef, second: ObjectRef) {
        self.conspire_mut()
            .instances
            .iter_mut()
            .find(|i| i.source == source && i.pay == Some(true) && !i.paid)
            .unwrap()
            .paid = true;
        self.tap_objects(&[first.card, second.card]);
    }
    pub(super) fn pay_conspire_printed(&mut self) {
        let mut a = self.state.announcing.take().unwrap();
        self.pay_additional_cost(&mut a);
        a.additional_paid = true;
        self.state.announcing = Some(a);
    }
    fn conspire_done(&self, a: &Announcement) -> bool {
        a.payment.is_none()
            && a.additional_paid
            && self.conspire_view().is_some_and(|c| {
                c.costs_started && c.instances.iter().all(|i| i.pay != Some(true) || i.paid)
            })
    }
    pub(super) fn defer_conspire_payment(&mut self, a: &mut Announcement) -> bool {
        if !self.conspire_has_payment() {
            return false;
        }
        if a.additional_cost().is_none() {
            a.additional_paid = true;
        }
        self.conspire_mut().costs_started = true;
        if self.conspire_projecting() || !self.conspire_done(a) {
            self.state.pending = Pending::ConspirePayment {
                player: a.item.controller,
            };
            return true;
        }
        false
    }
    pub(super) fn advance_conspire_payment(&mut self) -> bool {
        if !self.conspire_view().is_some_and(|c| c.costs_started)
            || self.state.entry_batch.is_some()
            || self.state.replacements.is_some()
        {
            return false;
        }
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        if self.conspire_done(a) && !self.conspire_projecting() {
            self.finish_announcement();
            return true;
        }
        self.state.pending = Pending::ConspirePayment {
            player: a.item.controller,
        };
        false
    }
    pub(super) fn visit_conspire_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let c = self.conspire_view().unwrap();
        if matches!(self.state.pending, Pending::ConspireIntent { .. }) {
            let instance = c.instances.iter().find(|i| i.pay.is_none()).unwrap();
            for pay in [false, true] {
                if emit(Action::ChooseConspire {
                    source: instance.source,
                    pay,
                }) {
                    return true;
                }
            }
        } else {
            let a = self.state.announcing.as_ref().unwrap();
            if !a.additional_paid
                && self.conspire_printed_valid(a)
                && emit(Action::PayConspirePrintedCosts)
            {
                return true;
            }
            for instance in c
                .instances
                .iter()
                .filter(|i| i.pay == Some(true) && !i.paid)
            {
                for first in 0..self.state.objects.len() {
                    for second in (first + 1)..self.state.objects.len() {
                        let (first, second) = (
                            self.object_ref(first as CardId),
                            self.object_ref(second as CardId),
                        );
                        if self.conspire_pair_valid(first, second)
                            && emit(Action::PayConspire {
                                source: instance.source,
                                first,
                                second,
                            })
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    fn conspire_printed_valid(&self, a: &Announcement) -> bool {
        a.additional_cost().is_none_or(|spec| {
            a.additional_group
                .as_deref()
                .map(Vec::as_slice)
                .unwrap_or_else(|| a.additional.as_slice())
                .iter()
                .all(|&r| self.additional_candidate(spec, a.item.controller, a.item.object, r))
        })
    }
    pub(super) fn conspire_fundable(&self, announcement: &Announcement, cost: ManaCost) -> bool {
        let mut child = self.clone();
        let mut a = announcement.clone();
        if a.cost != Some(cost) {
            a.total_cost = None;
        }
        a.cost = Some(cost);
        child.state.announcing = Some(a);
        child.conspire_mut().projecting = true;
        for i in &mut child.conspire_mut().instances {
            if i.pay.is_none() {
                i.pay = Some(false);
            }
        }
        child.continue_announcement_choices(false);
        child.conspire_completion_exists()
    }
    pub(super) fn conspire_action_viable(&self, action: &Action) -> bool {
        if *action == Action::Concede || self.conspire_projecting() {
            return true;
        }
        let Some(c) = self.conspire_view() else {
            return true;
        };
        let Some(a) = self.state.announcing.as_ref() else {
            return true;
        };
        if a.item.id != c.stack_id {
            return true;
        }
        // Earlier optional declarations do not yet have fixed target/cost domains.
        if !matches!(
            self.state.pending,
            Pending::ConspireIntent { .. }
                | Pending::ConspirePayment { .. }
                | Pending::PayCosts { .. }
        ) && self.state.entry_batch.is_none()
            && self.state.replacements.is_none()
        {
            return true;
        }
        if !self.conspire_has_payment()
            && !matches!(action, Action::ChooseConspire { pay: true, .. })
        {
            return true;
        }
        let mut child = self.clone();
        child.conspire_mut().projecting = true;
        let actor = child.actor().unwrap();
        if child.apply(actor, action.clone()).is_err() {
            return false;
        }
        if child.state.announcing.is_none() {
            return true;
        }
        for i in &mut child.conspire_mut().instances {
            if i.pay.is_none() {
                i.pay = Some(false);
            }
        }
        if matches!(child.state.pending, Pending::ConspireIntent { .. }) {
            child.continue_announcement_choices(false);
        }
        child.conspire_completion_exists()
    }
    fn conspire_upper_allows(&self, a: &Announcement, c: &ConspireCastView) -> bool {
        // Mana abilities can recolor sources, and removing characteristic Auras
        // can expose other colors or creature types. Linked costs can return new
        // objects. Those rare worlds require the exact completion search.
        if !self.state.exile_links.is_empty()
            || self.state.objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && (o.card.activations().iter().any(|ability| {
                        ability.mana_production().is_some_and(|p| {
                            matches!(p.base(), ManaProduction::AnyColorChangingSource(_))
                        })
                    }) || o.card.static_ability_groups().iter().any(|g| {
                        g.fragments.iter().any(|f| {
                            matches!(
                                f,
                                StaticAbilityFragment::AttachedCharacteristics(_)
                                    | StaticAbilityFragment::AttachedChosenColor
                            )
                        })
                    }))
            })
        {
            return true;
        }
        let required = c
            .instances
            .iter()
            .filter(|i| i.pay == Some(true) && !i.paid)
            .count()
            * 2;
        let available = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && !o.tapped
                    && self.is_creature(*id as CardId)
                    && ability_world::CharacteristicWorld::colors(self, *id as CardId)
                        & a.item.source_card.colors()
                        != 0
            })
            .count();
        available >= required
    }
    fn conspire_completion_exists(self) -> bool {
        fn visit(game: Game, seen: &mut Vec<Snapshot>) -> bool {
            let Some(a) = &game.state.announcing else {
                return true;
            };
            if game.state.entry_batch.is_none()
                && game.state.replacements.is_none()
                && game.conspire_done(a)
            {
                return true;
            }
            let c = game.conspire_view().unwrap();
            if !game.conspire_upper_allows(a, c) {
                return false;
            }
            let snapshot = game.snapshot();
            if seen.contains(&snapshot) {
                return false;
            }
            seen.push(snapshot);
            let Some(actor) = game.actor() else {
                return false;
            };
            game.legal_actions()
                .into_iter()
                .filter(|a| *a != Action::Concede)
                .any(|action| {
                    let mut child = game.clone();
                    child.apply(actor, action).is_ok() && visit(child, seen)
                })
        }
        visit(self, &mut Vec::new())
    }
    pub(super) fn capture_conspire_triggers(&mut self, index: usize) {
        let spell = self.state.stack[index].clone();
        let Some(c) = self.state.roles.as_mut().and_then(|r| r.conspire.take()) else {
            return;
        };
        if c.stack_id != spell.id {
            return;
        }
        for instance in c.instances.iter().filter(|i| i.pay == Some(true) && i.paid) {
            if !self.conspire_provider(instance.source, spell.controller, instance.ability_origin) {
                continue;
            }
            let mut trigger = spell.clone();
            trigger.id = self.next_stack_id();
            trigger.kind = StackKind::Conspire;
            trigger.targets.clear();
            trigger.cast = None;
            trigger.mode = None;
            trigger.cast_event = None;
            trigger.mana_copy_sources = None;
            trigger.copy_event = Some(Box::new(stack_copies::CopyEvent {
                spell: spell.clone(),
                amount: 1,
            }));
            self.state.pending_triggers.push(trigger);
        }
    }
}
