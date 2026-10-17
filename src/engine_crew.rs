//! Non-targeting Crew costs and sparse exact-incarnation layer-4 animation.
use super::saddle::PowerSum;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CrewContext {
    pub contributors: Vec<ObjectRef>,
    /// None means the exact signed sum is outside i128, not that it was clamped.
    pub paid_power: Option<i128>,
}
impl CrewContext {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.contributors.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CrewHistory {
    pub vehicle: ObjectRef,
    /// Distinct exact creatures that paid a Crew cost this turn, even if countered.
    pub contributors: Vec<ObjectRef>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CrewChoiceView {
    pub source: ObjectRef,
    pub minimum: u16,
    pub selected: Vec<ObjectRef>,
    pub selected_power: Option<i128>,
    pub can_finish: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CrewPayment {
    minimum: u16,
    selected: Vec<ObjectRef>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CrewState {
    pub animated: Vec<ObjectRef>,
    pub history: Vec<CrewHistory>,
    pub payment: Option<Box<CrewPayment>>,
}
impl CrewState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.animated.len() * std::mem::size_of::<ObjectRef>()
            + self.history.len() * std::mem::size_of::<CrewHistory>()
            + self
                .history
                .iter()
                .map(|h| h.contributors.len())
                .sum::<usize>()
                * std::mem::size_of::<ObjectRef>()
            + self.payment.as_ref().map_or(0, |p| {
                std::mem::size_of::<CrewPayment>()
                    + p.selected.len() * std::mem::size_of::<ObjectRef>()
            })
    }
}
impl ObjectView {
    pub fn is_creature(&self) -> bool {
        self.kinds & 1 != 0
    }
}
impl DamageFacts {
    pub fn is_creature(self) -> bool {
        self.kinds & 1 != 0
    }
}
impl ActivatedAbility {
    pub(super) fn crew_minimum(&self) -> Option<u16> {
        match self.program {
            [Instruction::Crew { minimum }] => Some(*minimum),
            _ => None,
        }
    }
}
impl Game {
    /// One borrowed query serves real objects and speculative funding overlays.
    pub(super) fn object_kinds(&self, id: CardId, object: &Object) -> u16 {
        let mut kinds = if object.zone == Zone::Stack {
            self.effective_spell_card(id).kind_mask()
        } else {
            object.card.kind_mask()
        };
        if object.zone == Zone::Battlefield
            && self.state.crew.as_ref().is_some_and(|c| {
                c.animated
                    .binary_search(&ObjectRef {
                        card: id,
                        generation: object.generation,
                    })
                    .is_ok()
            })
        {
            kinds |= 17;
        }
        kinds
    }
    pub(super) fn current_kinds(&self, id: CardId) -> u16 {
        self.object_kinds(id, &self.state.objects[id as usize])
    }
    pub(super) fn is_creature(&self, id: CardId) -> bool {
        self.current_kinds(id) & 1 != 0
    }
    fn crew_eligible(&self, source: ObjectRef, actor: Player, r: ObjectRef) -> bool {
        r != source
            && self.state.objects.get(r.card as usize).is_some_and(|o| {
                o.zone == Zone::Battlefield
                    && o.generation == r.generation
                    && o.controller == actor
                    && !o.tapped
                    && self.object_kinds(r.card, o) & 1 != 0
            })
    }
    pub(super) fn crew_fundable(&self, source: ObjectRef, actor: Player, minimum: u16) -> bool {
        if minimum == 0 {
            return true;
        }
        let mut sum = PowerSum::default();
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.crew_eligible(source, actor, r) {
                sum.add(self.stats(r.card).power.max(0));
                if sum.enough(minimum) {
                    return true;
                }
            }
        }
        false
    }
    fn crew_valid(&self) -> bool {
        let Some(a) = &self.state.announcing else {
            return false;
        };
        let Some(p) = self.state.crew.as_ref().and_then(|c| c.payment.as_ref()) else {
            return false;
        };
        self.state
            .objects
            .get(a.item.card as usize)
            .is_some_and(|o| {
                o.zone == Zone::Battlefield
                    && o.generation == a.item.object.generation
                    && o.controller == a.item.controller
            })
            && p.selected
                .iter()
                .all(|&r| self.crew_eligible(a.item.object, a.item.controller, r))
    }
    fn crew_sum(&self, maximum: bool) -> PowerSum {
        let a = self.state.announcing.as_ref().unwrap();
        let p = self.state.crew.as_ref().unwrap().payment.as_ref().unwrap();
        let mut sum = PowerSum::default();
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.crew_eligible(a.item.object, a.item.controller, r) {
                let selected = p.selected.contains(&r);
                let power = self.stats(r.card).power;
                if selected || maximum && power > 0 {
                    sum.add(power)
                }
            }
        }
        sum
    }
    pub(super) fn crew_choice(&self, viewer: Player) -> Option<CrewChoiceView> {
        if !matches!(self.state.pending,Pending::Crew{player} if player==viewer) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let p = self.state.crew.as_ref()?.payment.as_ref()?;
        let sum = self.crew_sum(false);
        Some(CrewChoiceView {
            source: a.item.object,
            minimum: p.minimum,
            selected: p.selected.clone(),
            selected_power: sum.represented(),
            can_finish: self.crew_valid() && sum.enough(p.minimum),
        })
    }
    pub(super) fn visit_crew_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        if !self.crew_valid() {
            return false;
        }
        let a = self.state.announcing.as_ref().unwrap();
        let p = self.state.crew.as_ref().unwrap().payment.as_ref().unwrap();
        let maximum = self.crew_sum(true);
        for id in 0..self.state.objects.len() {
            let r = self.object_ref(id as CardId);
            if self.crew_eligible(a.item.object, a.item.controller, r) && !p.selected.contains(&r) {
                let mut possible = maximum;
                let power = self.stats(r.card).power;
                if power < 0 {
                    possible.add(power)
                }
                if possible.enough(p.minimum) && emit(Action::ChooseCrewer { object: r }) {
                    return true;
                }
            }
        }
        self.crew_sum(false).enough(p.minimum) && emit(Action::FinishCrew)
    }
    pub(super) fn start_crew_payment(&mut self) -> bool {
        let Some(a) = &self.state.announcing else {
            return false;
        };
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return false;
        };
        let Some(minimum) = a.item.source_card.activations()[ability as usize].crew_minimum()
        else {
            return false;
        };
        assert!(a.pay_window && a.item.targets.is_empty());
        let state = self.state.crew.get_or_insert_with(Default::default);
        assert!(state.payment.is_none());
        state.payment = Some(Box::new(CrewPayment {
            minimum,
            selected: Vec::new(),
        }));
        self.state.pending = Pending::Crew {
            player: a.item.controller,
        };
        true
    }
    pub(super) fn choose_crewer(&mut self, object: ObjectRef) {
        let p = self.state.crew.as_mut().unwrap().payment.as_mut().unwrap();
        p.selected
            .try_reserve(1)
            .capacity("Crew selection capacity exhausted");
        p.selected.push(object);
    }
    pub(super) fn finish_crew_payment(&mut self) {
        assert!(self.crew_valid());
        let sum = self.crew_sum(false);
        let p = self.state.crew.as_mut().unwrap().payment.take().unwrap();
        assert!(sum.enough(p.minimum));
        let mut a = self.state.announcing.take().unwrap();
        let context = CrewContext {
            contributors: p.selected,
            paid_power: sum.represented(),
        };
        let state = self.state.crew.as_mut().unwrap();
        let at = state
            .history
            .iter()
            .position(|h| h.vehicle == a.item.object)
            .unwrap_or_else(|| {
                state
                    .history
                    .try_reserve(1)
                    .capacity("Crew history capacity exhausted");
                state.history.push(CrewHistory {
                    vehicle: a.item.object,
                    contributors: Vec::new(),
                });
                state.history.len() - 1
            });
        let history = &mut state.history[at].contributors;
        for &r in &context.contributors {
            if !history.contains(&r) {
                history
                    .try_reserve(1)
                    .capacity("Crew history capacity exhausted");
                history.push(r)
            }
        }
        self.pay_activation(&mut a.item, [0; 6], Counters::default());
        self.tap_objects(
            &context
                .contributors
                .iter()
                .map(|r| r.card)
                .collect::<Vec<_>>(),
        );
        a.item.crew = Some(Box::new(context));
        self.state.priority = a.item.controller;
        self.push_stack_item(a.item);
        self.state.pending = Pending::Priority;
        self.state.passes = 0;
    }
    pub(super) fn forget_vehicle_animation(&mut self, object: ObjectRef) {
        if let Some(state) = &mut self.state.crew
            && let Ok(at) = state.animated.binary_search(&object)
        {
            state.animated.remove(at);
        }
    }
    pub(super) fn expire_vehicle_animation(&mut self) -> bool {
        let Some(state) = &mut self.state.crew else {
            return false;
        };
        let changed = !state.animated.is_empty();
        state.animated.clear();
        changed
    }
    pub(super) fn clear_crew_history(&mut self) {
        if let Some(state) = &mut self.state.crew {
            state.history.clear();
            if state.animated.is_empty() && state.payment.is_none() {
                self.state.crew = None;
            }
        }
    }
    pub(super) fn resolve_crew(&mut self, item: &SpellView) {
        if !self.on_battlefield(item.card) || self.object_ref(item.card) != item.object {
            return;
        }
        let state = self.state.crew.get_or_insert_with(Default::default);
        if let Err(at) = state.animated.binary_search(&item.object) {
            state
                .animated
                .try_reserve(1)
                .capacity("Vehicle animation capacity exhausted");
            state.animated.insert(at, item.object)
        }
    }
}

#[cfg(test)]
mod core_tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    const NAMES: [&str; 24] = [
        "Adventurer's Airship",
        "Air Response Unit",
        "Boosted Sloop",
        "Broadcast Rambler",
        "Burner Rocket",
        "Careening Mine Cart",
        "Clamorous Ironclad",
        "Cultivator's Caravan",
        "Debris Beetle",
        "Dependable Quinjet",
        "Fire Nation Warship",
        "Haunted Hellride",
        "Hulldrifter",
        "Lumbering Worldwagon",
        "Magitek Armor",
        "Ripclaw Wrangler",
        "S.H.I.E.L.D. Helicarrier",
        "Skybox Ferry",
        "Spotcycle Scouter",
        "Strixhaven Skycoach",
        "Thopter Fabricator",
        "Tundra Tank",
        "Turtle Blimp",
        "Veloheart Bike",
    ];
    fn replay(g: &mut Game, action: Action) {
        let actor = g.actor().unwrap();
        let before = g.snapshot();
        let revision = g.revision();
        let mut branch = g.clone();
        branch.apply(actor, action.clone()).unwrap();
        g.apply(actor, action.clone()).unwrap();
        assert_eq!(g.snapshot(), branch.snapshot());
        let after = g.snapshot();
        g.restore(&before).unwrap();
        assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
        g.apply(actor, action).unwrap();
        assert_eq!(g.snapshot(), after);
    }
    fn settle(g: &mut Game) {
        g.prepare_triggers();
        for _ in 0..150 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.announcing.is_none()
            {
                return;
            }
            let choices = g.legal_actions();
            let action = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                choices[crate::policy::smoke_index(g, &choices)].clone()
            };
            replay(g, action);
        }
        panic!("unsettled {:?}", g.state.pending)
    }
    fn begin(g: &mut Game, vehicle: CardId) {
        let ability = g.state.objects[vehicle as usize]
            .card
            .activations()
            .iter()
            .position(|a| a.crew_minimum().is_some())
            .unwrap() as u16;
        replay(
            g,
            Action::AnnounceActivate {
                card: vehicle,
                ability,
                counter_payment: Counters::default(),
            },
        );
        assert!(matches!(g.state.pending, Pending::Crew { .. }));
    }
    fn contribute(g: &mut Game, id: CardId) {
        replay(
            g,
            Action::ChooseCrewer {
                object: g.object_ref(id),
            },
        );
    }
    #[test]
    fn all_twenty_four_actual_paid_casts_and_crew_keep_intrinsic_identity_both_seats() {
        for name in NAMES {
            for actor in [Player::P0, Player::P1] {
                let mut g = scenario();
                g.state.active = actor;
                g.state.priority = actor;
                g.state.players[actor.index()].mana = [30; 6];
                let contributor = spawn(&mut g, actor, Card::GrizzlyBears, Zone::Battlefield);
                g.state.objects[contributor as usize].growth.power = 8;
                g.state.objects[contributor as usize].summoning_sick = true;
                let card = Card::from_name(name).unwrap();
                assert!(card.is_supported(), "{name}");
                assert!(!card.is_creature());
                let source = spawn(&mut g, actor, card, Zone::Hand);
                let action=g.legal_actions().into_iter().find(|a| matches!(a,Action::Cast{card,..}|Action::AnnounceCast{card,..} if *card==source)).unwrap_or_else(||panic!("cast {name}"));
                replay(&mut g, action);
                settle(&mut g);
                assert!(g.on_battlefield(source), "{name}");
                assert!(!g.is_creature(source));
                assert_eq!(g.stats(source), Stats::default());
                let reference = g.object_ref(source);
                g.state.priority = actor;
                begin(&mut g, source);
                assert!(g.observe(actor).crew_choice.is_some());
                assert!(g.observe(actor.other()).crew_choice.is_none());
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a,Action::ChooseCrewer{object} if object.card==source))
                );
                contribute(&mut g, contributor);
                replay(&mut g, Action::FinishCrew);
                assert!(g.state.objects[contributor as usize].tapped);
                assert!(!g.state.objects[source as usize].tapped);
                assert!(!g.is_creature(source));
                let packet = g.state.stack.last().unwrap();
                assert_eq!(
                    packet.crew.as_ref().unwrap().contributors,
                    vec![g.object_ref(contributor)]
                );
                settle(&mut g);
                assert!(g.is_creature(source));
                assert_eq!(g.object_ref(source), reference);
                let body = g.object_view(source);
                let printed = card.vehicle_stats().unwrap();
                let expected = if name == "Lumbering Worldwagon" {
                    g.state
                        .objects
                        .iter()
                        .filter(|o| {
                            o.owner == actor && o.zone == Zone::Battlefield && o.card.is_land()
                        })
                        .count() as i128
                } else {
                    printed.0
                };
                assert_eq!(
                    (body.power, body.toughness),
                    (expected, printed.1),
                    "{name}"
                );
                assert!(!card.is_creature());
            }
        }
    }
    #[test]
    fn countered_crew_paid_history_and_exact_source_departure_do_not_animate_reentry() {
        for actor in [Player::P0, Player::P1] {
            let mut g = scenario();
            g.state.active = actor;
            g.state.priority = actor;
            let source = spawn(
                &mut g,
                actor,
                Card::from_name("Air Response Unit").unwrap(),
                Zone::Battlefield,
            );
            let first = spawn(&mut g, actor, Card::GrizzlyBears, Zone::Battlefield);
            begin(&mut g, source);
            contribute(&mut g, first);
            replay(&mut g, Action::FinishCrew);
            let packet = g.state.stack.pop().unwrap(); // Countering removes the packet; paid costs stay.
            assert!(!g.is_creature(source));
            assert!(g.state.objects[first as usize].tapped);
            assert_eq!(
                g.observe(actor).crewed_vehicles[0].contributors,
                vec![g.object_ref(first)]
            );
            let second = spawn(&mut g, actor, Card::GrizzlyBears, Zone::Battlefield);
            g.state.priority = actor;
            begin(&mut g, source);
            contribute(&mut g, second);
            replay(&mut g, Action::FinishCrew);
            let old = g.object_ref(source);
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
            assert_ne!(old, g.object_ref(source));
            settle(&mut g);
            assert!(!g.is_creature(source));
            assert_eq!(g.observe(actor).crewed_vehicles[0].contributors.len(), 2);
            assert_eq!(packet.crew.unwrap().contributors, vec![g.object_ref(first)]);
        }
    }
    #[test]
    fn negative_zero_and_excess_crewers_repeated_animation_and_self_exclusion() {
        let mut g = scenario();
        let source = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Air Response Unit").unwrap(),
            Zone::Battlefield,
        );
        let high = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        let zero = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        g.state.objects[zero as usize].growth.power = -2;
        let negative = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        g.state.objects[negative as usize].growth.power = -3;
        begin(&mut g, source);
        contribute(&mut g, zero);
        assert!(!g.legal_actions().contains(&Action::FinishCrew));
        contribute(&mut g, high);
        assert!(g.legal_actions().contains(&Action::FinishCrew));
        contribute(&mut g, negative);
        replay(&mut g, Action::FinishCrew);
        assert_eq!(
            g.state
                .stack
                .last()
                .unwrap()
                .crew
                .as_ref()
                .unwrap()
                .paid_power,
            Some(1)
        );
        settle(&mut g);
        for id in [high, zero, negative] {
            assert!(g.state.objects[id as usize].tapped)
        }
        g.state.objects[high as usize].tapped = false;
        g.state.priority = Player::P0;
        begin(&mut g, source);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::ChooseCrewer{object} if object.card==source))
        );
        contribute(&mut g, high);
        replay(&mut g, Action::FinishCrew);
        settle(&mut g);
        assert!(g.is_creature(source));
        assert_eq!(
            g.observe(Player::P0).crewed_vehicles[0].contributors.len(),
            3
        );
    }
    #[test]
    fn mana_funding_uses_live_sickness_and_never_hypothetically_crews() {
        for name in [
            "Cultivator's Caravan",
            "Dependable Quinjet",
            "Veloheart Bike",
        ] {
            for actor in [Player::P0, Player::P1] {
                let mut g = scenario();
                g.state.active = actor;
                g.state.priority = actor;
                let source = spawn(
                    &mut g,
                    actor,
                    Card::from_name(name).unwrap(),
                    Zone::Battlefield,
                );
                g.state.objects[source as usize].summoning_sick = true;
                let goal = funding::FundingGoal {
                    actor,
                    required: [1, 0, 0, 0, 0, 0],
                    generic: 0,
                    reserve: None,
                    source: None,
                };
                assert!(g.can_fund(goal.clone()));
                assert!(
                    g.legal_actions()
                        .iter()
                        .any(|a| matches!(a,Action::ActivateMana{card,..} if *card==source))
                );
                let contributor = spawn(&mut g, actor, Card::GrizzlyBears, Zone::Battlefield);
                g.state.objects[contributor as usize].growth.power = 6;
                begin(&mut g, source);
                // Crew is a pure tap-power cost, not a mana-payment opportunity.
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::ActivateMana { .. } | Action::TapMana { .. }))
                );
                contribute(&mut g, contributor);
                replay(&mut g, Action::FinishCrew);
                settle(&mut g);
                g.state.priority = actor;
                assert!(!g.can_fund(goal.clone()));
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a,Action::ActivateMana{card,..} if *card==source))
                );
                g.state.objects[source as usize].gained_keywords = Keywords::HASTE;
                assert!(g.can_fund(goal.clone()));
                g.state.objects[source as usize].gained_keywords = Keywords::NONE;
                g.state.objects[source as usize].summoning_sick = false;
                assert!(g.can_fund(goal.clone()));
                g.state.objects[source as usize].tapped = true;
                assert!(!g.can_fund(goal.clone()));
            }
        }
    }
    #[test]
    fn raw_vehicle_entry_ignores_creature_suppression_but_obeys_artifact_suppression() {
        for blocker in ["Torpor Orb", "Doorkeeper Thrull"] {
            for actor in [Player::P0, Player::P1] {
                let mut g = scenario();
                g.state.active = actor;
                g.state.priority = actor;
                g.state.players[actor.index()].mana = [10; 6];
                spawn(
                    &mut g,
                    actor.other(),
                    Card::from_name(blocker).unwrap(),
                    Zone::Battlefield,
                );
                let source = spawn(
                    &mut g,
                    actor,
                    Card::from_name("Hulldrifter").unwrap(),
                    Zone::Hand,
                );
                let before = g.state.players[actor.index()].cards_drawn_this_turn;
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a,Action::Cast{card,..} if *card==source))
                    .unwrap();
                replay(&mut g, action);
                settle(&mut g);
                assert_eq!(
                    g.state.players[actor.index()].cards_drawn_this_turn - before,
                    if blocker == "Torpor Orb" { 2 } else { 0 }
                );
                assert!(!g.is_creature(source));
            }
        }
    }
    #[test]
    fn real_cleanup_detaches_equipment_kills_creature_aura_preserves_counters_and_turn_history() {
        let mut g = scenario();
        let source = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Air Response Unit").unwrap(),
            Zone::Battlefield,
        );
        let contributor = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        begin(&mut g, source);
        contribute(&mut g, contributor);
        replay(&mut g, Action::FinishCrew);
        settle(&mut g);
        g.state.objects[source as usize].counters.plus_one = 2;
        let sword = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Short Bow").unwrap(),
            Zone::Battlefield,
        );
        assert!(g.attach_equipment(g.object_ref(sword), g.object_ref(source)));
        let aura = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Dead Weight").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
        g.state.objects[source as usize].damage = 4;
        g.state.step = Step::Cleanup;
        let turn = g.state.turn;
        g.finish_cleanup();
        assert!(!g.is_creature(source));
        assert_eq!(g.stats(source), Stats::default());
        assert_eq!(g.state.objects[source as usize].damage, 0);
        assert_eq!(g.state.objects[source as usize].counters.plus_one, 2);
        assert_eq!(g.object_view(sword).attached_to, None);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert_eq!(g.state.turn, turn);
        assert_eq!(g.observe(Player::P0).crewed_vehicles.len(), 1);
        assert_eq!(g.state.step, Step::Cleanup);
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert!(g.state.turn > turn);
        assert!(g.observe(Player::P0).crewed_vehicles.is_empty());
        // Printed stats return on the same incarnation, and counters persisted.
        g.state.crew = Some(Box::new(CrewState {
            animated: vec![g.object_ref(source)],
            ..Default::default()
        }));
        assert_eq!(g.stats(source).power, 5);
    }
    #[test]
    fn animated_death_lki_is_creature_but_intrinsic_graveyard_and_uncrewed_death_are_not() {
        for animated in [false, true] {
            for actor in [Player::P0, Player::P1] {
                let mut g = scenario();
                g.state.active = actor;
                g.state.priority = actor;
                let source = spawn(
                    &mut g,
                    actor,
                    Card::from_name("Fire Nation Warship").unwrap(),
                    Zone::Battlefield,
                );
                if animated {
                    let contributor = spawn(&mut g, actor, Card::GrizzlyBears, Zone::Battlefield);
                    begin(&mut g, source);
                    contribute(&mut g, contributor);
                    replay(&mut g, Action::FinishCrew);
                    settle(&mut g);
                }
                let captured = g.object_view(source);
                assert_eq!(captured.is_creature(), animated);
                let before = g.state.creatures_died_this_turn;
                let enemy = actor.other();
                g.state.priority = enemy;
                g.state.players[enemy.index()].mana = [8; 6];
                let card = Card::from_name(if animated { "Murder" } else { "Disenchant" }).unwrap();
                let spell = spawn(&mut g, enemy, card, Zone::Hand);
                let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Cast{card,target:Some(Target::Card(t)),..} if *card==spell&&*t==source)).unwrap();
                replay(&mut g, a);
                settle(&mut g);
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
                assert!(!g.object_view(source).is_creature());
                assert_eq!(
                    g.state.creatures_died_this_turn,
                    before + u64::from(animated)
                );
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.owner == actor
                            && o.zone == Zone::Battlefield
                            && o.card.is_token()
                            && o.card.has_subtype("Clue"))
                        .count(),
                    1
                );
                assert_eq!(captured.kinds & 1, u16::from(animated));
                assert_eq!(g.object_view(source).kinds, 16);
            }
        }
    }
    #[test]
    fn exact_wide_crew_sum_and_late_capacity_restore_paid_history_taps_and_private_frame() {
        let mut g = scenario();
        let source = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Air Response Unit").unwrap(),
            Zone::Battlefield,
        );
        let mut high = Vec::new();
        for _ in 0..2 {
            let id = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
            g.state.objects[id as usize].growth.power = i128::MAX - 2;
            high.push(id);
        }
        let negative = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
        g.state.objects[negative as usize].growth.power = i128::MIN;
        let observer = spawn(
            &mut g,
            Player::P0,
            Card::from_name("Attentive Sunscribe").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[observer as usize].growth.power = -g.stats(observer).power;
        begin(&mut g, source);
        contribute(&mut g, negative);
        assert!(!g.legal_actions().contains(&Action::FinishCrew));
        contribute(&mut g, high[0]);
        assert_eq!(
            g.observe(Player::P0).crew_choice.unwrap().selected_power,
            Some(1)
        );
        contribute(&mut g, high[1]);
        let frame = g.observe(Player::P0).crew_choice.unwrap();
        assert_eq!(frame.selected_power, None);
        assert!(frame.can_finish);
        contribute(&mut g, observer);
        // Actual tap-trigger allocation fails only after the cost batch taps.
        let original = g.state.next_stack_id;
        g.state.next_stack_id = u64::MAX;
        let before = g.snapshot();
        let revision = g.revision();
        let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
        assert!(matches!(
            g.apply(Player::P0, Action::FinishCrew),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        for (n, p) in [Player::P0, Player::P1].into_iter().enumerate() {
            assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[n]);
        }
        assert!(!g.state.objects[negative as usize].tapped);
        assert!(g.observe(Player::P0).crewed_vehicles.is_empty());
        g.state.next_stack_id = original;
        replay(&mut g, Action::FinishCrew);
        assert_eq!(
            g.state
                .stack
                .iter()
                .find(|item| item.crew.is_some())
                .unwrap()
                .crew
                .as_ref()
                .unwrap()
                .paid_power,
            None
        );
        settle(&mut g);
        assert!(g.is_creature(source));
    }
    #[test]
    fn core_live_kind_is_exact_and_printed_vehicle_identity_is_immutable() {
        let mut g = scenario();
        let card = Card::from_name("Air Response Unit").unwrap();
        let id = spawn(&mut g, Player::P0, card, Zone::Battlefield);
        let old = g.object_ref(id);
        assert!(!g.object_view(id).is_creature());
        assert_eq!(g.stats(id), Stats::default());
        g.state.crew = Some(Box::new(CrewState {
            animated: vec![old],
            ..Default::default()
        }));
        assert!(g.object_view(id).is_creature());
        assert!(!card.is_creature());
        assert_eq!((g.stats(id).power, g.stats(id).toughness), (3, 3));
        let captured = g.object_view(id);
        let facts = g.damage_facts(id);
        let saved = g.snapshot();
        let mut child = g.clone();
        child.move_object(id, Zone::Hand);
        assert!(!child.object_view(id).is_creature());
        assert!(captured.is_creature() && facts.is_creature());
        child.move_object(id, Zone::Battlefield);
        assert!(!child.object_view(id).is_creature());
        assert_ne!(child.object_ref(id), old);
        assert!(g.object_view(id).is_creature());
        child.restore(&saved).unwrap();
        assert!(child.object_view(id).is_creature());
    }

    #[test]
    fn composed_paid_creature_target_and_counter_replacements_use_animated_vehicle_kinds() {
        for p in [Player::P0, Player::P1] {
            for loading_first in [false, true] {
                let mut g = scenario();
                g.state.active = p;
                g.state.priority = p;
                g.state.players[p.index()].mana = [30; 6];
                let mut named = |name, zone| spawn(&mut g, p, Card::from_name(name).unwrap(), zone);
                let vehicle = named("Cultivator's Caravan", Zone::Battlefield);
                let a = named("Bear Cub", Zone::Battlefield);
                let b = named("Bear Cub", Zone::Battlefield);
                let michelangelo = named("Michelangelo, Weirdness to 11", Zone::Battlefield);
                let loading = named("Loading Zone", Zone::Battlefield);
                let spell = named("Snakeskin Veil", Zone::Hand);
                let cast = Action::Cast {
                    card: spell,
                    target: Some(Target::Card(vehicle)),
                    payment: [0, 0, 0, 0, 1, 0],
                };
                let before = g.snapshot();
                let revision = g.revision();
                assert_eq!(g.apply(p, cast.clone()), Err(Error::IllegalAction));
                assert_eq!(g.snapshot(), before);
                assert_eq!(g.revision(), revision);
                begin(&mut g, vehicle);
                contribute(&mut g, a);
                contribute(&mut g, b);
                replay(&mut g, Action::FinishCrew);
                settle(&mut g);
                assert!(g.is_creature(vehicle));
                assert!(!g.state.objects[vehicle as usize].card.is_creature());
                replay(&mut g, cast);
                let packet = g.state.stack.last().unwrap();
                assert_eq!(packet.cast.unwrap().payment, [0, 0, 0, 0, 1, 0]);
                replay(&mut g, Action::Pass);
                replay(&mut g, Action::Pass);
                let frame = g
                    .observe(p)
                    .replacement_choice
                    .expect("both live creature filters apply");
                assert_eq!(frame.effects.len(), 2);
                let public = g
                    .observe(p.other())
                    .replacement_choice
                    .expect("battlefield counter event is public");
                assert_eq!(
                    serde_json::to_value(&frame).unwrap(),
                    serde_json::to_value(&public).unwrap()
                );
                let first = if loading_first { loading } else { michelangelo };
                let choice = g.legal_actions().into_iter().find(|a| matches!(a,Action::ChooseReplacement { source,.. } if source.card == first)).unwrap();
                let before = g.snapshot();
                let revision = g.revision();
                assert!(g.apply(p.other(), choice.clone()).is_err());
                assert_eq!(g.snapshot(), before);
                assert_eq!(g.revision(), revision);
                let mut child = {
                    let parent = g.clone();
                    let child = parent.clone();
                    drop(parent);
                    child
                };
                child.apply(p, choice.clone()).unwrap();
                g.apply(p, choice.clone()).unwrap();
                assert_eq!(g.snapshot(), child.snapshot());
                g.restore(&before).unwrap();
                g.apply(p, choice).unwrap();
                assert_eq!(g.snapshot(), child.snapshot());
                settle(&mut g);
                settle(&mut child);
                assert_eq!(g.snapshot(), child.snapshot());
                assert_eq!(
                    g.state.objects[vehicle as usize].counters.plus_one,
                    if loading_first { 3 } else { 4 }
                );
                assert!(g.keywords(vehicle).contains(Keywords::HEXPROOF));
            }
        }
    }
}
