//! Intrinsic convoke: exact matching, a closed mana window, and atomic tap costs.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvokeContribution {
    Generic,
    White,
    Blue,
    Black,
    Red,
    Green,
}
impl ConvokeContribution {
    fn color(self) -> Option<usize> {
        match self {
            Self::Generic => None,
            Self::White => Some(0),
            Self::Blue => Some(1),
            Self::Black => Some(2),
            Self::Red => Some(3),
            Self::Green => Some(4),
        }
    }
    fn apply(self, colors: u8, cost: &mut ManaCost) -> bool {
        if let Some(i) = self.color() {
            if colors & (1 << i) == 0 || cost.colored[i] == 0 {
                return false;
            }
            cost.colored[i] -= 1;
        } else {
            if cost.generic == 0 {
                return false;
            }
            cost.generic -= 1;
        }
        true
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConvokePaymentView {
    pub object: ObjectRef,
    pub contribution: ConvokeContribution,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConvokeSelection {
    pub selected: Vec<ConvokePaymentView>,
    pub remaining: ManaCost,
}
impl ConvokeSelection {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.selected.len() * std::mem::size_of::<ConvokePaymentView>()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConvokeChoiceView {
    pub spell: ObjectRef,
    pub selected: Vec<ConvokePaymentView>,
    pub remaining: ManaCost,
    pub can_pay: bool,
}

/// Hall's theorem on the five-color bipartite matching problem. Only 32 masks
/// exist, so neither creature subsets nor numeric mana units are enumerated.
/// Real mana covers matching requirements first; exchanges preserve feasibility.
/// Colorless creatures pay generic only; required {C} always consumes real mana.
pub(super) fn payable(pool: [u128; 6], required: Mana, generic: u32, masks: &[u64; 32]) -> bool {
    if pool[5] < u128::from(required[5]) {
        return false;
    }
    let deficits: [u64; 5] = std::array::from_fn(|i| {
        u64::try_from(u128::from(required[i]).saturating_sub(pool[i])).unwrap()
    });
    let mut subset = *masks;
    for bit in 0..5 {
        for mask in 0..32 {
            if mask & (1 << bit) != 0 {
                subset[mask] += subset[mask ^ (1 << bit)];
            }
        }
    }
    let total = subset[31];
    let mut needs = [0u64; 32];
    for mask in 1usize..32 {
        let i = mask.trailing_zeros() as usize;
        needs[mask] = needs[mask ^ (1 << i)] + deficits[i];
        if needs[mask] > total - subset[31 ^ mask] {
            return false;
        }
    }
    let mut left = u128::from(generic).saturating_sub(u128::from(total - needs[31]));
    for i in 0..6 {
        let spare = pool[i].saturating_sub(u128::from(required[i]));
        if spare >= left {
            return true;
        }
        left -= spare;
    }
    left == 0
}
impl Game {
    fn convoke_eligible(&self, p: Player, r: ObjectRef) -> bool {
        self.state.objects.get(r.card as usize).is_some_and(|o| {
            o.generation == r.generation
                && o.owner == p
                && o.zone == Zone::Battlefield
                && o.card.is_creature()
                && !o.tapped
        })
    }
    pub(super) fn convoke_masks(&self, p: Player, selected: &[ConvokePaymentView]) -> [u64; 32] {
        let mut masks = [0; 32];
        for (id, o) in self.state.objects.iter().enumerate() {
            let r = self.object_ref(id as CardId);
            if self.convoke_eligible(p, r) && !selected.iter().any(|s| s.object == r) {
                masks[usize::from(o.card.colors() & 31)] += 1;
            }
        }
        masks
    }
    pub fn convoke_choice(&self) -> Option<ConvokeChoiceView> {
        if !matches!(self.state.pending, Pending::Convoke { .. }) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let f = a.convoke.as_ref()?;
        let pool = self.state.players[a.item.controller.index()].mana;
        let can_pay = payable(
            pool.map(u128::from),
            f.remaining.colored,
            f.remaining.generic,
            &[0; 32],
        );
        Some(ConvokeChoiceView {
            spell: a.item.object,
            selected: f.selected.clone(),
            remaining: f.remaining,
            can_pay,
        })
    }
    pub(super) fn begin_convoke(&mut self) {
        let a = self.state.announcing.as_mut().unwrap();
        assert!(
            a.item.kind == StackKind::Spell
                && a.item.source_card.has_convoke()
                && a.convoke.is_none()
        );
        a.convoke = Some(Box::new(ConvokeSelection {
            remaining: a.total_cost.expect("locked total"),
            selected: Vec::new(),
        }));
        self.state.pending = Pending::Convoke {
            player: a.item.controller,
        };
    }
    pub(super) fn choose_convoker(&mut self, object: ObjectRef, contribution: ConvokeContribution) {
        let colors = self.state.objects[object.card as usize].card.colors();
        let f = self
            .state
            .announcing
            .as_mut()
            .unwrap()
            .convoke
            .as_mut()
            .unwrap();
        assert!(contribution.apply(colors, &mut f.remaining));
        f.selected.push(ConvokePaymentView {
            object,
            contribution,
        });
    }
    pub(super) fn visit_convoke_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let f = a.convoke.as_ref().unwrap();
        let p = a.item.controller;
        // Exact selected incarnations remain reserved until this atomic cost ends.
        if !f
            .selected
            .iter()
            .all(|s| self.convoke_eligible(p, s.object))
        {
            return false;
        }
        let pool = self.state.players[p.index()].mana;
        let mut done = false;
        payments(pool, f.remaining.colored, f.remaining.generic, |payment| {
            if !done {
                done = emit(Action::PayCosts { payment });
            }
        });
        if done {
            return true;
        }
        let masks = self.convoke_masks(p, &f.selected);
        for (id, o) in self.state.objects.iter().enumerate() {
            let object = self.object_ref(id as CardId);
            if !self.convoke_eligible(p, object) || f.selected.iter().any(|s| s.object == object) {
                continue;
            }
            let colors = o.card.colors();
            let mut rest = masks;
            rest[usize::from(colors & 31)] -= 1;
            for contribution in [
                ConvokeContribution::Generic,
                ConvokeContribution::White,
                ConvokeContribution::Blue,
                ConvokeContribution::Black,
                ConvokeContribution::Red,
                ConvokeContribution::Green,
            ] {
                let mut cost = f.remaining;
                if contribution.apply(colors, &mut cost)
                    && payable(pool.map(u128::from), cost.colored, cost.generic, &rest)
                    && emit(Action::ChooseConvoker {
                        object,
                        contribution,
                    })
                {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{scenario, spawn};
    use super::*;
    const NAMES: [&str; 9] = [
        "Appeal to Eirdu",
        "Disruptor of Currents",
        "Lofty Dreams",
        "Protective Response",
        "Sun-Dappled Celebrant",
        "Unexpected Assistance",
        "Vote Out",
        "Winter, Team Player",
        "Merrow Skyswimmer",
    ];
    pub(super) fn replay(g: &mut Game, a: Action) {
        let before = g.snapshot();
        let actor = g.actor().unwrap();
        let revision = g.revision();
        let mut clone = g.clone();
        clone.apply(actor, a.clone()).unwrap();
        g.apply(actor, a.clone()).unwrap();
        assert_eq!(clone.snapshot(), g.snapshot());
        let after = g.snapshot();
        g.restore(&before).unwrap();
        assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
        g.apply(actor, a).unwrap();
        assert_eq!(g.snapshot(), after);
    }
    pub(super) fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
        spawn(g, p, Card::from_name(n).unwrap(), z)
    }
    pub(super) fn settle(g: &mut Game) {
        for _ in 0..60 {
            if g.state.stack.is_empty()
                && g.state.announcing.is_none()
                && g.state.resolving.is_none()
                && g.state.pending == Pending::Priority
            {
                return;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::FinishTargets
                            | Action::Discard { .. }
                            | Action::PutTrigger { .. }
                            | Action::Pass
                    )
                })
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::ChooseTarget { .. }))
                })
                .unwrap_or_else(|| panic!("{:?}", g.legal_actions()));
            replay(g, a);
        }
        panic!("did not settle");
    }
    #[test]
    fn hall_matching_equals_exhaustive_individual_assignments_and_extreme_bounds() {
        fn brute(pool: [u128; 6], mut req: Mana, generic: u32, creatures: &[u8]) -> bool {
            if creatures.is_empty() {
                return payable(pool, req, generic, &[0; 32]);
            }
            let colors = creatures[0];
            let rest = &creatures[1..];
            if brute(pool, req, generic, rest) || generic > 0 && brute(pool, req, generic - 1, rest)
            {
                return true;
            }
            for i in 0..5 {
                if colors & (1 << i) != 0 && req[i] > 0 {
                    req[i] -= 1;
                    if brute(pool, req, generic, rest) {
                        return true;
                    }
                    req[i] += 1;
                }
            }
            false
        }
        for a in 0..32u8 {
            for b in 0..32u8 {
                for white in 0..=2 {
                    for blue in 0..=2 {
                        for generic in 0..=2 {
                            let req = [white, blue, 0, 0, 0, 0];
                            let pool = [0, 1, 0, 0, 0, 0];
                            let mut masks = [0; 32];
                            masks[a as usize] += 1;
                            masks[b as usize] += 1;
                            assert_eq!(
                                payable(pool, req, generic, &masks),
                                brute(pool, req, generic, &[a, b]),
                                "{a} {b} {req:?} {generic}"
                            );
                        }
                    }
                }
            }
        }
        let mut masks = [0; 32];
        masks[31] = u64::from(u32::MAX) * 5;
        masks[0] = u64::from(u32::MAX);
        assert!(payable(
            [0; 6],
            [u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, 0],
            u32::MAX,
            &masks
        ));
        assert!(!payable([0; 6], [0, 0, 0, 0, 0, 1], 0, &masks));
        assert!(payable([u128::MAX; 6], [u32::MAX; 6], u32::MAX, &[0; 32]));
    }
    #[test]
    fn all_nine_pay_real_creature_costs_with_zero_pool_both_controllers_and_full_hybrid_options() {
        for name in NAMES {
            for p in [Player::P0, Player::P1] {
                for hybrid in 0..if name == "Merrow Skyswimmer" { 3 } else { 1 } {
                    let mut g = scenario();
                    g.state.active = p;
                    g.state.priority = p;
                    let spell = named(&mut g, p, name, Zone::Hand);
                    let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                    // Attacking/blocking is independent of the convoke contributor choice.
                    g.state.attackers.push(Attacker {
                        card: victim,
                        defender: Some(CombatDefender::Player(p)),
                        blocked: false,
                    });
                    let mut colors = Vec::new();
                    for color in 0..5 {
                        let c = Card::catalogue()
                            .find(|c| {
                                c.is_supported()
                                    && c.is_creature()
                                    && c.colors() == 1 << color
                                    && c.triggers().is_empty()
                                    && !c.is_legendary()
                            })
                            .unwrap();
                        for _ in 0..6 {
                            let id = spawn(&mut g, p, c, Zone::Battlefield);
                            g.state.objects[id as usize].summoning_sick = true;
                            colors.push(id);
                        }
                    }
                    replay(&mut g, Action::AnnounceCast { card: spell });
                    if matches!(g.state.pending, Pending::SpellCost { .. }) {
                        replay(&mut g, Action::ChooseSpellCost { cost: hybrid });
                    }
                    while matches!(g.state.pending, Pending::Targets { .. }) {
                        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ChooseTarget{target:Target::Card(id)} if *id==victim)).or_else(||g.legal_actions().into_iter().find(|a|matches!(a,Action::FinishTargets))).unwrap();
                        replay(&mut g, a);
                    }
                    let locked = g.state.announcing.as_ref().unwrap().total_cost.unwrap();
                    replay(&mut g, Action::BeginConvoke);
                    assert!(g.observe(p.other()).convoke_choice.is_none());
                    assert!(g.observe(p.other()).payment_request.is_none());
                    let mut tapped = Vec::new();
                    while !g.convoke_choice().unwrap().can_pay {
                        let a = g
                            .legal_actions()
                            .into_iter()
                            .find(|a| matches!(a, Action::ChooseConvoker { .. }))
                            .unwrap();
                        if let Action::ChooseConvoker { object, .. } = a {
                            assert!(!g.state.objects[object.card as usize].tapped);
                            tapped.push(object.card);
                        }
                        replay(&mut g, a);
                        assert!(g.legal_actions().iter().all(|a| !matches!(
                            a,
                            Action::TapMana { .. }
                                | Action::ActivateMana { .. }
                                | Action::BeginConvoke
                        )));
                    }
                    assert_eq!(
                        g.state.announcing.as_ref().unwrap().total_cost,
                        Some(locked)
                    );
                    assert_eq!(
                        tapped.len() as u32,
                        locked.generic + locked.colored.iter().sum::<u32>()
                    );
                    assert!(
                        tapped
                            .iter()
                            .all(|&id| !g.state.objects[id as usize].tapped)
                    );
                    assert!(
                        !g.legal_actions()
                            .iter()
                            .any(|a| matches!(a, Action::ChooseConvoker { .. }))
                    );
                    replay(&mut g, Action::PayCosts { payment: [0; 6] });
                    assert!(tapped.iter().all(|&id| g.state.objects[id as usize].tapped));
                    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                    assert_eq!(g.state.stack.last().unwrap().source_card.name(), name);
                    settle(&mut g);
                    assert_eq!(
                        g.state.objects[spell as usize].zone,
                        if Card::from_name(name).unwrap().is_creature() || name == "Lofty Dreams" {
                            Zone::Battlefield
                        } else {
                            Zone::Graveyard
                        }
                    );
                    if name == "Merrow Skyswimmer" {
                        assert!(g.state.objects.iter().any(|o| {
                            o.owner == p
                                && o.zone == Zone::Battlefield
                                && o.card.token_info().is_some_and(|t| {
                                    t.subtypes == "Merfolk" && o.card.colors() == 3
                                })
                        }));
                    }
                }
            }
        }
    }
    #[test]
    fn colors_ownership_duplicates_closed_window_and_zero_overpayment_are_exact() {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, "Sun-Dappled Celebrant", Zone::Hand);
        let white = named(&mut g, Player::P0, "Merrow Skyswimmer", Zone::Battlefield);
        let other = named(&mut g, Player::P1, "Merrow Skyswimmer", Zone::Battlefield);
        g.state.players[0].mana = [1, 0, 0, 0, 0, 4];
        replay(&mut g, Action::AnnounceCast { card: spell });
        replay(&mut g, Action::BeginConvoke);
        let r = g.object_ref(white);
        let before = g.snapshot();
        for (object, contribution) in [
            (g.object_ref(other), ConvokeContribution::White),
            (r, ConvokeContribution::Green),
            (
                ObjectRef {
                    generation: r.generation + 1,
                    ..r
                },
                ConvokeContribution::White,
            ),
        ] {
            assert_eq!(
                g.apply(
                    Player::P0,
                    Action::ChooseConvoker {
                        object,
                        contribution
                    }
                ),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), before);
        }
        // Generic would consume the sole remaining white contributor: correctly absent.
        assert_eq!(
            g.apply(
                Player::P0,
                Action::ChooseConvoker {
                    object: r,
                    contribution: ConvokeContribution::Generic
                }
            ),
            Err(Error::IllegalAction)
        );
        replay(
            &mut g,
            Action::ChooseConvoker {
                object: r,
                contribution: ConvokeContribution::White,
            },
        );
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseConvoker { .. }))
        );
        let a = Action::PayCosts {
            payment: [1, 0, 0, 0, 0, 4],
        };
        assert_eq!(g.apply(Player::P1, a.clone()), Err(Error::WrongActor));
        replay(&mut g, a);
    }
    #[test]
    fn virtual_mana_and_convoke_never_double_spend_a_tapped_or_sacrificed_creature() {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, "Sun-Dappled Celebrant", Zone::Hand);
        let elves = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        g.state.players[0].mana = [2, 0, 0, 0, 0, 2];
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceCast{card} if *card==spell))
        );
        named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        replay(&mut g, Action::AnnounceCast { card: spell });
        replay(&mut g, Action::TapMana { card: elves });
        replay(&mut g, Action::BeginConvoke);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::ChooseConvoker{object,..} if object.card==elves))
        );
        let choice = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseConvoker { .. }))
            .unwrap();
        replay(&mut g, choice);
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, pay);
        assert_eq!(g.state.players[0].mana, [0; 6]);
    }
}

#[cfg(test)]
mod residual_tests {
    use super::super::tests::{scenario, spawn};
    use super::tests::{named, replay, settle};
    use super::*;
    #[test]
    fn real_tap_triggers_queue_above_cast_and_late_capacity_restores_the_entire_frame() {
        for p in [Player::P0, Player::P1] {
            for fail in [false, true] {
                let mut g = scenario();
                g.state.active = p;
                g.state.priority = p;
                g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 3];
                let spell = named(&mut g, p, "Sun-Dappled Celebrant", Zone::Hand);
                let helper = named(&mut g, p, "Chrome Companion", Zone::Battlefield);
                g.state.objects[helper as usize].summoning_sick = true;
                replay(&mut g, Action::AnnounceCast { card: spell });
                replay(&mut g, Action::BeginConvoke);
                let object = g.object_ref(helper);
                replay(
                    &mut g,
                    Action::ChooseConvoker {
                        object,
                        contribution: ConvokeContribution::Generic,
                    },
                );
                let payment = [2, 0, 0, 0, 0, 3];
                let action = Action::PayCosts { payment };
                if fail {
                    g.state.next_stack_id = u64::MAX;
                    let before = g.snapshot();
                    let revision = g.revision();
                    let views = [
                        serde_json::to_value(g.observe(p)).unwrap(),
                        serde_json::to_value(g.observe(p.other())).unwrap(),
                    ];
                    for _ in 0..2 {
                        assert!(matches!(
                            g.apply(p, action.clone()),
                            Err(Error::CapacityExceeded(_))
                        ));
                        assert_eq!(g.snapshot(), before);
                        assert_eq!(g.revision(), revision);
                        assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[0]);
                        assert_eq!(
                            serde_json::to_value(g.observe(p.other())).unwrap(),
                            views[1]
                        );
                        assert!(!g.object_view(helper).tapped);
                    }
                } else {
                    replay(&mut g, action);
                    assert!(g.object_view(helper).tapped);
                    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                    assert_eq!(g.state.stack[0].source_card.name(), "Sun-Dappled Celebrant");
                    assert_eq!(g.state.stack.last().unwrap().card, helper);
                    settle(&mut g);
                    assert_eq!(g.state.players[p.index()].life, 21);
                }
            }
        }
    }
    #[test]
    fn zero_contributors_and_real_mana_only_are_legal_but_convoke_does_not_pay_activations() {
        for begin in [false, true] {
            let mut g = scenario();
            let spell = named(&mut g, Player::P0, "Sun-Dappled Celebrant", Zone::Hand);
            let helper = named(&mut g, Player::P0, "Chrome Companion", Zone::Battlefield);
            g.state.players[0].mana = [2, 0, 0, 0, 0, 4];
            replay(&mut g, Action::AnnounceCast { card: spell });
            if begin {
                replay(&mut g, Action::BeginConvoke);
                assert!(g.convoke_choice().unwrap().can_pay);
            }
            replay(
                &mut g,
                Action::PayCosts {
                    payment: [2, 0, 0, 0, 0, 4],
                },
            );
            assert!(!g.object_view(helper).tapped);
            assert_eq!(g.state.players[0].mana, [0; 6]);
            settle(&mut g);
        }
        let mut g = scenario();
        let body = named(
            &mut g,
            Player::P0,
            "Mystic Archaeologist",
            Zone::Battlefield,
        );
        for _ in 0..10 {
            named(&mut g, Player::P0, "Merrow Skyswimmer", Zone::Battlefield);
        }
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceActivate{card,..} if *card==body))
        );
        g.state.players[0].mana = [0, 2, 0, 0, 0, 3];
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: body,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert!(!g.legal_actions().contains(&Action::BeginConvoke));
    }
    #[test]
    fn exact_incarnation_reservation_and_owned_frame_memory_do_not_leak_or_rebind() {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, "Sun-Dappled Celebrant", Zone::Hand);
        let helper = named(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
        g.state.players[0].mana = [1, 0, 0, 0, 0, 4];
        replay(&mut g, Action::AnnounceCast { card: spell });
        let before = g.state_bytes();
        replay(&mut g, Action::BeginConvoke);
        let empty = g.state_bytes();
        assert_eq!(empty - before, std::mem::size_of::<ConvokeSelection>());
        let object = g.object_ref(helper);
        replay(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::White,
            },
        );
        assert_eq!(
            g.state_bytes() - empty,
            std::mem::size_of::<ConvokePaymentView>()
        );
        assert!(g.observe(Player::P1).convoke_choice.is_none());
        assert!(g.observe(Player::P1).actions.is_empty());
        let saved = g.snapshot();
        g.move_object(helper, Zone::Graveyard);
        g.move_object(helper, Zone::Battlefield);
        assert_ne!(object, g.object_ref(helper));
        assert_eq!(g.legal_actions(), vec![Action::Concede]);
        g.restore(&saved).unwrap();
        replay(
            &mut g,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 0, 4],
            },
        );
    }
    #[test]
    fn appeal_requires_one_or_two_distinct_targets_and_resolves_each_surviving_incarnation() {
        for keep in [true, false] {
            let mut g = scenario();
            g.state.players[0].mana = [1, 0, 0, 0, 0, 3];
            let spell = named(&mut g, Player::P0, "Appeal to Eirdu", Zone::Hand);
            let a = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
            replay(&mut g, Action::AnnounceCast { card: spell });
            assert!(!g.legal_actions().contains(&Action::FinishTargets));
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            assert!(g.legal_actions().contains(&Action::FinishTargets));
            assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(a)
            }));
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(b),
                },
            );
            replay(
                &mut g,
                Action::PayCosts {
                    payment: [1, 0, 0, 0, 0, 3],
                },
            );
            g.move_object(a, Zone::Graveyard);
            g.move_object(a, Zone::Battlefield);
            if !keep {
                g.move_object(b, Zone::Graveyard);
                g.move_object(b, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.stats(a),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert_eq!(
                g.stats(b),
                if keep {
                    Stats {
                        power: 4,
                        toughness: 3,
                    }
                } else {
                    Stats {
                        power: 2,
                        toughness: 2,
                    }
                }
            );
        }
    }
    #[test]
    fn disruptor_and_bespoke_other_target_is_optional_any_controller_nonland_and_equip_is_complete()
    {
        for name in ["Disruptor of Currents", "Bespoke Bō"] {
            for p in [Player::P0, Player::P1] {
                for target_owner in [None, Some(p), Some(p.other())] {
                    let mut g = scenario();
                    g.state.active = p;
                    g.state.priority = p;
                    g.state.players[p.index()].mana = [20; 6];
                    let source = named(&mut g, p, name, Zone::Hand);
                    let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                    let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                    let land = spawn(&mut g, p.other(), Card::Forest, Zone::Battlefield);
                    replay(&mut g, Action::AnnounceCast { card: source });
                    let pay = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayCosts { .. }))
                        .unwrap();
                    replay(&mut g, pay);
                    for _ in 0..10 {
                        if matches!(g.state.pending, Pending::Targets { .. }) {
                            break;
                        }
                        let action = g
                            .legal_actions()
                            .into_iter()
                            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                            .unwrap();
                        replay(&mut g, action);
                    }
                    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Card(source)
                    }));
                    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Card(land)
                    }));
                    assert!(g.legal_actions().contains(&Action::FinishTargets));
                    if let Some(owner) = target_owner {
                        replay(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(if owner == p { a } else { b }),
                            },
                        );
                    } else {
                        replay(&mut g, Action::FinishTargets);
                    }
                    settle(&mut g);
                    if let Some(owner) = target_owner {
                        assert_eq!(
                            g.object_view(if owner == p { a } else { b }).zone,
                            Zone::Hand
                        );
                    }
                    if name == "Bespoke Bō" {
                        let wearer = if target_owner == Some(p) {
                            named(&mut g, p, "Bear Cub", Zone::Battlefield)
                        } else {
                            a
                        };
                        g.state.priority = p;
                        replay(
                            &mut g,
                            Action::AnnounceActivate {
                                card: source,
                                ability: 0,
                                counter_payment: Counters::default(),
                            },
                        );
                        replay(
                            &mut g,
                            Action::ChooseTarget {
                                target: Target::Card(wearer),
                            },
                        );
                        let pay = g
                            .legal_actions()
                            .into_iter()
                            .find(|a| matches!(a, Action::PayCosts { .. }))
                            .unwrap();
                        replay(&mut g, pay);
                        settle(&mut g);
                        assert_eq!(
                            g.stats(wearer),
                            Stats {
                                power: 4,
                                toughness: 3
                            }
                        );
                        assert!(g.keywords(wearer).contains(Keywords::VIGILANCE));
                        g.move_object(source, Zone::Hand);
                        assert_eq!(
                            g.stats(wearer),
                            Stats {
                                power: 2,
                                toughness: 2
                            }
                        );
                        assert!(!g.keywords(wearer).contains(Keywords::VIGILANCE));
                    }
                }
            }
        }
    }
}
