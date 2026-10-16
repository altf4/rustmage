//! Incremental non-targeting tap costs and exact, resolved Saddle designations.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SaddlePayment {
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SaddleChoiceView {
    pub source: ObjectRef,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    /// None only if the exact signed aggregate is outside i128; legality stays exact.
    pub selected_power: Option<i128>,
    pub can_finish: bool,
}
/// Each sign sums magnitudes with a u64 carry above u128. CardId bounds the
/// number of terms, so the high limb cannot overflow. Cancellation is exact.
#[derive(Clone, Copy, Default)]
struct PowerSum {
    positive: (u64, u128),
    negative: (u64, u128),
}
impl PowerSum {
    fn add(&mut self, p: i128) {
        let limb = if p < 0 {
            &mut self.negative
        } else {
            &mut self.positive
        };
        let (low, carry) = limb.1.overflowing_add(p.unsigned_abs());
        limb.1 = low;
        limb.0 += u64::from(carry);
    }
    fn difference(self) -> (bool, u64, u128) {
        let neg = self.negative > self.positive;
        let (a, b) = if neg {
            (self.negative, self.positive)
        } else {
            (self.positive, self.negative)
        };
        let (low, borrow) = a.1.overflowing_sub(b.1);
        (neg, a.0 - b.0 - u64::from(borrow), low)
    }
    fn enough(self, n: u16) -> bool {
        let (neg, hi, lo) = self.difference();
        !neg && (hi > 0 || lo >= u128::from(n))
    }
    fn represented(self) -> Option<i128> {
        let (neg, hi, lo) = self.difference();
        if hi > 0 {
            return None;
        }
        if neg {
            if lo == 1u128 << 127 {
                Some(i128::MIN)
            } else {
                i128::try_from(lo).ok().map(|n| -n)
            }
        } else {
            i128::try_from(lo).ok()
        }
    }
}
impl ActivatedAbility {
    pub(super) fn saddle_minimum(&self) -> Option<u16> {
        match self.program {
            [Instruction::BecomeSaddled { minimum }] => Some(*minimum),
            _ => None,
        }
    }
}
impl Game {
    pub(super) fn is_saddled(&self, object: ObjectRef) -> bool {
        self.state.saddled.contains(&object)
    }
    fn saddle_eligible(&self, source: ObjectRef, p: Player, r: ObjectRef) -> bool {
        self.state.objects.get(r.card as usize).is_some_and(|o| {
            o.generation == r.generation
                && o.zone == Zone::Battlefield
                && o.owner == p
                && o.card.is_creature()
                && !o.tapped
                && r != source
        })
    }
    pub(super) fn saddle_fundable(&self, source: ObjectRef, p: Player, minimum: u16) -> bool {
        if minimum == 0 {
            return true;
        }
        let mut available = 0u16;
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.saddle_eligible(source, p, r) {
                let power = self.stats(r.card).power.max(0);
                available = available.saturating_add(u16::try_from(power).unwrap_or(u16::MAX));
                if available >= minimum {
                    return true;
                }
            }
        }
        false
    }
    fn saddle_frame_valid(&self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        let Some(f) = self.state.saddle_payment.as_ref() else {
            return false;
        };
        self.state
            .objects
            .get(a.item.object.card as usize)
            .is_some_and(|o| {
                o.generation == a.item.object.generation
                    && o.zone == Zone::Battlefield
                    && o.owner == a.item.controller
            })
            && f.selected
                .iter()
                .all(|r| self.saddle_eligible(a.item.object, a.item.controller, *r))
    }
    fn saddle_sums(&self, maximum: bool) -> PowerSum {
        let a = self.state.announcing.as_ref().unwrap();
        let f = self.state.saddle_payment.as_ref().unwrap();
        let mut sum = PowerSum::default();
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.saddle_eligible(a.item.object, a.item.controller, r) {
                let selected = f.selected.contains(&r);
                if selected || maximum {
                    let p = self.stats(r.card).power;
                    if selected || p > 0 {
                        sum.add(p);
                    }
                }
            }
        }
        sum
    }
    pub fn saddle_choice(&self) -> Option<SaddleChoiceView> {
        if !matches!(self.state.pending, Pending::Saddle { .. }) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let f = self.state.saddle_payment.as_ref()?;
        let sum = self.saddle_sums(false);
        Some(SaddleChoiceView {
            source: a.item.object,
            minimum: f.minimum,
            selected: f.selected.clone(),
            selected_power: sum.represented(),
            can_finish: self.saddle_frame_valid() && sum.enough(f.minimum),
        })
    }
    pub(super) fn visit_saddle_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let f = self.state.saddle_payment.as_ref().unwrap();
        if !self.saddle_frame_valid() {
            return false;
        }
        let maximum = self.saddle_sums(true);
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            if !self.saddle_eligible(a.item.object, a.item.controller, object)
                || f.selected.contains(&object)
            {
                continue;
            }
            let p = self.stats(object.card).power;
            let mut possible = maximum;
            if p < 0 {
                possible.add(p);
            }
            if possible.enough(f.minimum) && emit(Action::ChooseSaddler { object }) {
                return true;
            }
        }
        self.saddle_sums(false).enough(f.minimum) && emit(Action::FinishSaddle)
    }
    pub(super) fn start_saddle_payment(&mut self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return false;
        };
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return false;
        };
        let Some(minimum) = a.item.source_card.activations()[ability as usize].saddle_minimum()
        else {
            return false;
        };
        assert!(a.pay_window && a.item.targets.is_empty());
        assert!(self.state.saddle_payment.is_none());
        self.state.saddle_payment = Some(Box::new(SaddlePayment {
            minimum,
            selected: Vec::new(),
        }));
        self.state.pending = Pending::Saddle {
            player: a.item.controller,
        };
        true
    }
    pub(super) fn finish_saddle_payment(&mut self) {
        assert!(self.saddle_frame_valid());
        let f = self.state.saddle_payment.take().unwrap();
        let a = self.state.announcing.take().unwrap();
        assert!(f.selected.iter().all(|r| self.saddle_eligible(
            a.item.object,
            a.item.controller,
            *r
        )));
        let mut total = PowerSum::default();
        for r in &f.selected {
            total.add(self.stats(r.card).power);
        }
        assert!(total.enough(f.minimum));
        // All powers are checked before the simultaneous tap cost. Tap observers
        // queue during payment, then are ordered above the activated ability.
        self.pay_activation(&a.item, [0; 6], Counters::default());
        self.tap_objects(&f.selected.iter().map(|r| r.card).collect::<Vec<_>>());
        self.state.priority = a.item.controller;
        self.push_stack_item(a.item);
        self.state.pending = Pending::Priority;
        self.state.passes = 0;
    }
    pub(super) fn resolve_saddle(&mut self, item: &SpellView) {
        if !self.on_battlefield(item.card) || self.object_ref(item.card) != item.object {
            return;
        }
        let first = !self.is_saddled(item.object);
        if first {
            self.state.saddled.push(item.object);
        }
        if first {
            let source = self.object_view(item.card);
            for (index, a) in item.source_card.triggers().iter().enumerate() {
                if matches!(a.condition.event(), TriggerCondition::SelfSaddledFirst) {
                    self.queue_trigger(&source, index);
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    const NAMES: [&str; 19] = [
        "Alacrian Jaguar",
        "Autarch Mammoth",
        "Bounding Felidar",
        "Bridled Bighorn",
        "Brightfield Glider",
        "Brightfield Mustang",
        "Congregation Gryff",
        "District Mascot",
        "Dracosaur Auxiliary",
        "Drover Grizzly",
        "Gila Courser",
        "Gilded Ghoda",
        "Gloryheath Lynx",
        "Quilled Charger",
        "Seraphic Steed",
        "Stubborn Burrowfiend",
        "Trained Arynx",
        "Unswerving Sloth",
        "Venomsac Lagac",
    ];
    fn replay(g: &mut Game, a: Action) {
        let saved = g.snapshot();
        let actor = g.actor().unwrap();
        let revision = g.revision();
        let mut f = g.clone();
        f.apply(actor, a.clone()).unwrap();
        g.apply(actor, a.clone()).unwrap();
        assert_eq!(g.snapshot(), f.snapshot());
        let after = g.snapshot();
        g.restore(&saved).unwrap();
        assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
        g.apply(actor, a).unwrap();
        assert_eq!(g.snapshot(), after);
    }
    fn settle(g: &mut Game) {
        g.prepare_triggers();
        for _ in 0..100 {
            if g.state.announcing.is_none()
                && g.state.stack.is_empty()
                && g.state.pending == Pending::Priority
            {
                return;
            }
            let choices = g.legal_actions();
            let preferred=choices.iter().find(|a|matches!(a,Action::ChooseSearch{..}))
                .or_else(||choices.iter().find(|a|matches!(a,Action::ChooseTarget{target:Target::Player(p)} if *p==g.state.active.other())));
            let a = preferred
                .cloned()
                .or_else(|| {
                    choices.into_iter().find(|a| {
                        matches!(
                            a,
                            Action::Pass
                                | Action::PutTrigger { .. }
                                | Action::ChooseTarget { .. }
                                | Action::FinishTargets
                                | Action::FinishSearch
                                | Action::ArrangeLibrary { .. }
                        )
                    })
                })
                .unwrap_or_else(|| panic!("unhandled {:?}", g.legal_actions()));
            replay(g, a);
        }
        panic!("unsettled saddle");
    }
    #[test]
    fn all_nineteen_paid_cast_saddle_resolution_and_attack_packets_both_controllers() {
        for name in NAMES {
            for p in [Player::P0, Player::P1] {
                let mut g = scenario();
                g.state.active = p;
                g.state.priority = p;
                g.state.players[p.index()].mana = [30; 6];
                let plains = spawn(&mut g, p, Card::Plains, Zone::Library);
                let source = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Hand);
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a,Action::Cast{card,..} if *card==source))
                    .expect(name);
                replay(&mut g, a);
                settle(&mut g);
                assert!(g.on_battlefield(source), "{name}");
                let contributor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                g.state.objects[contributor as usize].growth.power = 8;
                g.state.objects[contributor as usize].summoning_sick = true;
                g.state.objects[source as usize].summoning_sick = false;
                g.state.priority = p;
                let ability = g.state.objects[source as usize]
                    .card
                    .activations()
                    .iter()
                    .position(|a| a.saddle_minimum().is_some())
                    .unwrap() as u16;
                replay(
                    &mut g,
                    Action::AnnounceActivate {
                        card: source,
                        ability,
                        counter_payment: Counters::default(),
                    },
                );
                assert_eq!(g.state.pending, Pending::Saddle { player: p });
                assert!(!g.is_saddled(g.object_ref(source)));
                assert!(!g.state.objects[contributor as usize].tapped);
                let r = g.object_ref(contributor);
                replay(&mut g, Action::ChooseSaddler { object: r });
                replay(&mut g, Action::FinishSaddle);
                assert!(g.state.objects[contributor as usize].tapped);
                assert!(!g.is_saddled(g.object_ref(source)));
                assert_eq!(
                    g.state.stack.last().unwrap().kind,
                    StackKind::ActivatedAbility { ability }
                );
                settle(&mut g);
                assert!(g.is_saddled(g.object_ref(source)), "{name}");
                g.state.step = Step::Attackers;
                g.state.pending = Pending::Attackers;
                g.state.active = p;
                g.state.priority = p;
                replay(&mut g, Action::Attack { card: source });
                replay(&mut g, Action::FinishAttackers);
                if name != "Stubborn Burrowfiend" {
                    assert!(
                        g.state
                            .pending_triggers
                            .iter()
                            .chain(g.state.ordering_triggers.iter())
                            .chain(g.state.stack.iter())
                            .any(|t| t.object == g.object_ref(source))
                            || g.state
                                .announcing
                                .as_ref()
                                .is_some_and(|a| a.item.object == g.object_ref(source)),
                        "{name}"
                    );
                }
                settle(&mut g);
                assert_eq!(
                    g.state.objects[source as usize].zone,
                    Zone::Battlefield,
                    "{name}"
                );
                let body = g.object_view(source);
                let tokens = |subtype: &str| {
                    g.state
                        .objects
                        .iter()
                        .filter(|o| {
                            o.owner == p
                                && o.zone == Zone::Battlefield
                                && o.card.is_token()
                                && o.card.has_subtype(subtype)
                        })
                        .count()
                };
                match name {
                    "Alacrian Jaguar" => assert_eq!((body.power, body.toughness), (6, 6)),
                    "Autarch Mammoth" => assert_eq!(tokens("Elephant"), 2),
                    "Bounding Felidar" => {
                        assert_eq!(g.object_view(contributor).counters.plus_one, 1);
                        assert_eq!(g.state.players[p.index()].life, 21);
                        assert_eq!(body.counters.plus_one, 0);
                    }
                    "Bridled Bighorn" => assert_eq!(tokens("Sheep"), 1),
                    "Brightfield Glider" => {
                        assert_eq!((body.power, body.toughness), (2, 3));
                        assert!(body.keywords.contains(Keywords::FLYING));
                    }
                    "Brightfield Mustang" => {
                        assert!(!body.tapped);
                        assert_eq!(body.counters.plus_one, 1);
                    }
                    "Congregation Gryff" => assert_eq!((body.power, body.toughness), (2, 5)),
                    "District Mascot" => {
                        assert_eq!((body.power, body.toughness), (2, 2));
                        assert_eq!(body.counters.plus_one, 2);
                    }
                    "Dracosaur Auxiliary" => {
                        assert_eq!(g.state.players[p.other().index()].life, 18)
                    }
                    "Drover Grizzly" => {
                        assert!(body.keywords.contains(Keywords::TRAMPLE));
                        assert!(g.keywords(contributor).contains(Keywords::TRAMPLE));
                    }
                    "Gila Courser" => assert_eq!(g.object_view(plains).zone, Zone::Exile),
                    "Gilded Ghoda" => assert_eq!(tokens("Treasure"), 1),
                    "Gloryheath Lynx" => assert_eq!(g.object_view(plains).zone, Zone::Hand),
                    "Quilled Charger" => {
                        assert_eq!((body.power, body.toughness), (5, 5));
                        assert!(body.keywords.contains(Keywords::MENACE));
                    }
                    "Seraphic Steed" => assert_eq!(tokens("Angel"), 1),
                    "Stubborn Burrowfiend" => assert_eq!((body.power, body.toughness), (2, 2)),
                    "Trained Arynx" => assert!(body.keywords.contains(Keywords::FIRST_STRIKE)),
                    "Unswerving Sloth" => {
                        assert!(body.keywords.contains(Keywords::INDESTRUCTIBLE));
                        assert!(!body.tapped);
                        assert!(!g.object_view(contributor).tapped);
                    }
                    "Venomsac Lagac" => assert_eq!((body.power, body.toughness), (2, 4)),
                    _ => unreachable!(),
                }
            }
        }
    }
    fn saddle(g: &mut Game, source: CardId, contributors: &[CardId]) {
        let ability = g.state.objects[source as usize]
            .card
            .activations()
            .iter()
            .position(|a| a.saddle_minimum().is_some())
            .unwrap() as u16;
        replay(
            g,
            Action::AnnounceActivate {
                card: source,
                ability,
                counter_payment: Counters::default(),
            },
        );
        for id in contributors {
            let object = g.object_ref(*id);
            replay(g, Action::ChooseSaddler { object });
        }
        replay(g, Action::FinishSaddle);
    }
    #[test]
    fn paid_tap_triggers_precede_saddle_and_countered_or_departed_sources_never_mark() {
        for p in [Player::P0, Player::P1] {
            for depart in [false, true] {
                let mut g = scenario();
                g.state.active = p;
                g.state.priority = p;
                let source = spawn(
                    &mut g,
                    p,
                    Card::from_name("Alacrian Jaguar").unwrap(),
                    Zone::Battlefield,
                );
                let companion = spawn(
                    &mut g,
                    p,
                    Card::from_name("Chrome Companion").unwrap(),
                    Zone::Battlefield,
                );
                g.state.objects[source as usize].tapped = true;
                g.state.objects[companion as usize].summoning_sick = true;
                saddle(&mut g, source, &[companion]);
                let id = g.state.stack[0].id;
                assert!(g.state.objects[companion as usize].tapped);
                assert!(!g.is_saddled(g.object_ref(source)));
                assert_eq!(g.state.stack.last().unwrap().card, companion);
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].life, 21);
                assert!(g.is_saddled(g.object_ref(source)));
                g.state.objects[companion as usize].tapped = false;
                g.state.priority = p;
                saddle(&mut g, source, &[companion]);
                let next = g.state.stack[0].id;
                assert_ne!(next, id);
                if depart {
                    let old = g.object_ref(source);
                    g.move_object(source, Zone::Graveyard);
                    g.move_object(source, Zone::Battlefield);
                    assert_ne!(old, g.object_ref(source));
                    assert!(!g.is_saddled(g.object_ref(source)));
                } else {
                    g.state.saddled.clear();
                    g.counter_stack_item(next, false);
                }
                settle(&mut g);
                assert!(g.state.objects[companion as usize].tapped);
                assert!(!g.is_saddled(g.object_ref(source)));
                assert_eq!(g.state.players[p.index()].life, 22);
            }
        }
    }
    #[test]
    fn stubborn_first_resolved_event_counts_newly_milled_cards_and_resets_at_cleanup() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Stubborn Burrowfiend").unwrap(),
                Zone::Battlefield,
            );
            let contributor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            spawn(&mut g, p, Card::GrizzlyBears, Zone::Library);
            spawn(&mut g, p, Card::GrizzlyBears, Zone::Library);
            let library = g.state.players[p.index()].library.len();
            saddle(&mut g, source, &[contributor]);
            settle(&mut g);
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            assert_eq!(g.state.players[p.index()].library.len(), library - 2);
            g.state.objects[contributor as usize].tapped = false;
            g.state.priority = p;
            saddle(&mut g, source, &[contributor]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), library - 2);
            g.state.step = Step::Cleanup;
            g.finish_cleanup();
            assert!(!g.is_saddled(g.object_ref(source)));
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
        }
    }
    #[test]
    fn exact_attack_packet_survives_departure_without_rebinding_source_effects() {
        for p in [Player::P0, Player::P1] {
            for name in ["Alacrian Jaguar", "Autarch Mammoth"] {
                let mut g = scenario();
                g.state.active = p;
                g.state.priority = p;
                let source = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Battlefield);
                let contributor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                g.state.objects[contributor as usize].growth.power = 8;
                saddle(&mut g, source, &[contributor]);
                settle(&mut g);
                g.state.step = Step::Attackers;
                g.state.pending = Pending::Attackers;
                replay(&mut g, Action::Attack { card: source });
                replay(&mut g, Action::FinishAttackers);
                let old = g.object_ref(source);
                g.move_object(source, Zone::Graveyard);
                g.move_object(source, Zone::Battlefield);
                assert_ne!(old, g.object_ref(source));
                assert!(!g.is_saddled(g.object_ref(source)));
                settle(&mut g);
                if name == "Alacrian Jaguar" {
                    assert_eq!(
                        g.stats(source),
                        Stats {
                            power: 4,
                            toughness: 4
                        }
                    );
                } else {
                    assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                        && o.card.is_token()
                        && o.card.has_subtype("Elephant")));
                }
            }
        }
    }
    #[test]
    fn selected_power_outside_i128_remains_exactly_finishable_and_stale_frames_offer_no_actions() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Alacrian Jaguar").unwrap(),
                Zone::Battlefield,
            );
            let mut contributors = Vec::new();
            for _ in 0..2 {
                let id = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                g.state.objects[id as usize].growth.power = i128::MAX - 2;
                contributors.push(id);
            }
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            for id in &contributors {
                let object = g.object_ref(*id);
                replay(&mut g, Action::ChooseSaddler { object });
            }
            let f = g.saddle_choice().unwrap();
            assert_eq!(f.selected_power, None);
            assert!(f.can_finish);
            let snapshot = g.snapshot();
            g.state.objects[contributors[0] as usize].generation += 1;
            assert!(!g.saddle_choice().unwrap().can_finish);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseSaddler { .. } | Action::FinishSaddle))
            );
            g.restore(&snapshot).unwrap();
            g.state.objects[source as usize].generation += 1;
            assert!(!g.saddle_choice().unwrap().can_finish);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseSaddler { .. } | Action::FinishSaddle))
            );
            g.restore(&snapshot).unwrap();
            replay(&mut g, Action::FinishSaddle);
            assert!(contributors.iter().all(|id| g.object_view(*id).tapped));
            settle(&mut g);
            assert!(g.is_saddled(g.object_ref(source)));
        }
    }
    #[test]
    fn district_mascot_retains_actual_entry_counter_and_targeted_remove_two_activation() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            g.state.players[p.index()].mana = [30; 6];
            let source = spawn(
                &mut g,
                p,
                Card::from_name("District Mascot").unwrap(),
                Zone::Hand,
            );
            super::super::tests::cast(&mut g, source, None);
            settle(&mut g);
            assert_eq!(g.object_view(source).counters.plus_one, 1);
            let target = spawn(
                &mut g,
                p.other(),
                Card::from_name("Candy Trail").unwrap(),
                Zone::Battlefield,
            );
            let contributor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            g.state.objects[source as usize].summoning_sick = false;
            g.state.priority = p;
            saddle(&mut g, source, &[contributor]);
            settle(&mut g);
            g.state.pending = Pending::Attackers;
            g.state.step = Step::Attackers;
            replay(&mut g, Action::Attack { card: source });
            replay(&mut g, Action::FinishAttackers);
            settle(&mut g);
            assert_eq!(g.object_view(source).counters.plus_one, 2);
            g.state.priority = p;
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters {
                        plus_one: 2,
                        ..Counters::default()
                    },
                },
            );
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            for _ in 0..20 {
                if g.state.announcing.is_none() {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::PayCosts { .. }
                                | Action::ChooseSpellCost { .. }
                                | Action::FinishTargets
                        )
                    })
                    .unwrap();
                replay(&mut g, a);
            }
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        }
    }
    #[test]
    fn failed_late_tap_trigger_identity_allocation_restores_selected_frame_costs_and_rng() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Alacrian Jaguar").unwrap(),
                Zone::Battlefield,
            );
            let contributor = spawn(
                &mut g,
                p,
                Card::from_name("Chrome Companion").unwrap(),
                Zone::Battlefield,
            );
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let object = g.object_ref(contributor);
            replay(&mut g, Action::ChooseSaddler { object });
            g.state.next_stack_id = u64::MAX;
            let saved = g.snapshot();
            let revision = g.revision();
            let views = [
                serde_json::to_value(g.observe(p)).unwrap(),
                serde_json::to_value(g.observe(p.other())).unwrap(),
            ];
            for _ in 0..2 {
                assert!(matches!(
                    g.apply(p, Action::FinishSaddle),
                    Err(Error::CapacityExceeded(_))
                ));
                assert_eq!(g.snapshot(), saved);
                assert_eq!(g.revision(), revision);
                assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[0]);
                assert_eq!(
                    serde_json::to_value(g.observe(p.other())).unwrap(),
                    views[1]
                );
                assert!(!g.object_view(contributor).tapped);
                assert!(!g.object_view(source).saddled);
            }
        }
    }
    #[test]
    fn saddle_obeys_sorcery_timing_without_a_source_tap_or_summoning_sickness_cost() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Alacrian Jaguar").unwrap(),
                Zone::Battlefield,
            );
            let contributor = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
            g.state.objects[source as usize].tapped = true;
            g.state.objects[source as usize].summoning_sick = true;
            g.state.objects[contributor as usize].summoning_sick = true;
            let action = Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            };
            assert!(g.legal_actions().contains(&action));
            g.state.step = Step::BeginCombat;
            assert!(!g.legal_actions().contains(&action));
            g.state.step = Step::Main1;
            g.state.active = p.other();
            assert!(!g.legal_actions().contains(&action));
            g.state.active = p;
            replay(&mut g, action);
            let object = g.object_ref(contributor);
            replay(&mut g, Action::ChooseSaddler { object });
            replay(&mut g, Action::FinishSaddle);
            settle(&mut g);
            assert!(g.object_view(source).tapped);
            assert!(g.object_view(source).saddled);
        }
    }
    #[test]
    fn signed_power_carries_cancel_exactly_without_query_overflow() {
        let mut s = PowerSum::default();
        s.add(i128::MAX);
        s.add(i128::MAX);
        assert_eq!(s.represented(), None);
        assert!(s.enough(u16::MAX));
        s.add(i128::MIN);
        assert_eq!(s.represented(), Some(i128::MAX - 1));
        s.add(i128::MIN);
        assert_eq!(s.represented(), Some(-2));
        assert!(!s.enough(0));
        s.add(2);
        assert_eq!(s.represented(), Some(0));
        assert!(s.enough(0));
        let mut s = PowerSum::default();
        for _ in 0..1024 {
            s.add(i128::MAX);
        }
        for _ in 0..1024 {
            s.add(-i128::MAX);
        }
        assert_eq!(s.represented(), Some(0));
    }
    #[test]
    fn saddle_incremental_domain_keeps_every_completable_zero_negative_and_extra_cost() {
        for p in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = p;
            g.state.priority = p;
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Axgard Cavalry").unwrap(),
                Zone::Battlefield,
            );
            g.state.objects[source as usize].tapped = true;
            let mut make = |power: i128| {
                let id = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                g.state.objects[id as usize].growth.power = power - 2;
                g.state.objects[id as usize].summoning_sick = true;
                id
            };
            let positive = make(6);
            let second = make(2);
            let negative = make(-2);
            let zero = make(0);
            let impossible = make(-5);
            let foreign = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            assert!(g.saddle_fundable(g.object_ref(source), p, 8));
            assert!(!g.saddle_fundable(g.object_ref(source), p, 9));
            let item = g.activation_item(p, source, 0);
            g.open_payment_announcement(item, Counters::default(), None);
            g.state.saddle_payment = Some(Box::new(SaddlePayment {
                minimum: 4,
                selected: Vec::new(),
            }));
            g.state.pending = Pending::Saddle { player: p };
            let ids = |g: &Game| {
                g.legal_actions()
                    .into_iter()
                    .filter_map(|a| {
                        if let Action::ChooseSaddler { object } = a {
                            Some(object.card)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(ids(&g), vec![positive, second, negative, zero]);
            assert!(!ids(&g).contains(&foreign));
            assert!(!ids(&g).contains(&source));
            assert!(!ids(&g).contains(&impossible));
            let before = g.snapshot();
            let r = g.revision();
            assert_eq!(
                g.apply(
                    p.other(),
                    Action::ChooseSaddler {
                        object: g.object_ref(positive)
                    }
                ),
                Err(Error::WrongActor)
            );
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), r);
            assert!(g.observe(p.other()).saddle_choice.is_none());
            assert!(!g.saddle_choice().unwrap().can_finish);
            for id in [negative, positive, zero, second] {
                let a = Action::ChooseSaddler {
                    object: g.object_ref(id),
                };
                let saved = g.snapshot();
                let mut fork = g.clone();
                fork.apply(p, a.clone()).unwrap();
                g.apply(p, a.clone()).unwrap();
                assert_eq!(fork.snapshot(), g.snapshot());
                let expected = g.snapshot();
                g.restore(&saved).unwrap();
                g.apply(p, a).unwrap();
                assert_eq!(g.snapshot(), expected);
            }
            let f = g.saddle_choice().unwrap();
            assert_eq!(f.selected_power, Some(6));
            assert!(f.can_finish);
            assert!(g.legal_actions().contains(&Action::FinishSaddle));
            assert!(!g.state.objects[positive as usize].tapped);
        }
    }
}
