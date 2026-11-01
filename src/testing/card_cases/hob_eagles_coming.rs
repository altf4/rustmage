//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid ownership-sensitive returns and one frozen next-upkeep token event per spell.
use super::*;
const FIXTURES: &[&str] = &[
    "The Eagles Are Coming!",
    "Bear Cub",
    "Forest",
    "Doubling Season",
    "Banishing Light",
    "Adaptive Automaton",
    "Negate",
    "@token/hob-wolf",
    "@token/hob-bird-soldier",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Eagles budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn mana(kicked: bool) -> Mana {
    if kicked {
        [3, 0, 0, 0, 0, 3]
    } else {
        [1, 0, 0, 0, 0, 1]
    }
}
fn announce_eagles(g: &mut Game, p: Player, kicked: bool, ids: &[CardId]) -> CardId {
    let spell = fixture(g, p, "The Eagles Are Coming!", Zone::Hand);
    g.state.players[p.index()].mana = mana(kicked);
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseKicker { pay: kicked });
    for &id in ids {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().kicker,
        Some(kicked)
    );
    spell
}
fn birds(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::from_name("@token/hob-bird-soldier").unwrap())
            .then_some(id as CardId)
        })
        .collect()
}
fn delayed(g: &Game) -> Vec<u64> {
    g.state
        .delayed_upkeep_tokens
        .as_ref()
        .into_iter()
        .flat_map(|r| r.iter())
        .map(|r| {
            if let StackKind::DelayedTokenCreation { amount, .. } = r.kind {
                amount
            } else {
                panic!("wrong delayed kind")
            }
        })
        .collect()
}
fn step(g: &mut Game, p: Player, s: Step) {
    g.state.active = p;
    g.begin_step(s);
    g.give_priority();
    g.prepare_triggers();
}
#[test]
fn eagles_paid_instant_owns_not_controls_return_and_next_any_player_upkeep_exact_recipe() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let own = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.record_control(g.object_ref(own), p.other(), None);
        g.refresh_controllers();
        let foreign = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.record_control(g.object_ref(foreign), p, None);
        g.refresh_controllers();
        g.state.active = p.other();
        g.state.priority = p;
        g.state.step = Step::BeginCombat;
        let spell = fixture(&mut g, p, "The Eagles Are Coming!", Zone::Hand);
        g.state.players[p.index()].mana = mana(false);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseKicker { pay: false });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        finish(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Hand);
        assert_eq!(g.object_view(own).owner, p);
        assert_eq!(g.object_view(foreign).zone, Zone::Battlefield);
        assert_eq!(delayed(&g), vec![1]);
        assert!(birds(&g, p).is_empty());
        step(&mut g, p.other(), Step::Main1);
        finish(&mut g);
        assert_eq!(delayed(&g), vec![1]);
        step(&mut g, p.other(), Step::Upkeep);
        finish(&mut g);
        let b = birds(&g, p);
        assert_eq!(b.len(), 1);
        let v = g.object_view(b[0]);
        assert_eq!((v.power, v.toughness), (4, 4));
        assert_eq!(v.colors, 1);
        assert!(v.has_subtype("Bird") && v.has_subtype("Soldier"));
        assert!(v.keywords.contains(Keywords::FLYING));
        assert_eq!(v.owner, p);
        assert!(!v.tapped);
        assert!(g.observe(p).delayed_upkeep_tokens.is_none());
        step(&mut g, p, Step::Upkeep);
        finish(&mut g);
        assert_eq!(birds(&g, p).len(), 1);
    }
}
#[test]
fn eagles_kicked_zero_targets_still_registers_one_zero_amount_delayed_trigger() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        announce_eagles(&mut g, p, true, &[]);
        finish(&mut g);
        assert_eq!(delayed(&g), vec![0]);
        step(&mut g, p.other(), Step::Upkeep);
        until(&mut g, |g| g.state.stack.len() == 1);
        assert!(matches!(
            g.state.stack[0].kind,
            StackKind::DelayedTokenCreation { amount: 0, .. }
        ));
        finish(&mut g);
        assert!(birds(&g, p).is_empty());
        assert!(delayed(&g).is_empty());
    }
}

#[test]
fn eagles_countered_spell_does_not_return_or_schedule_and_failed_kicker_payment_rolls_back() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    let spell = announce_eagles(&mut g, p, false, &[a]);
    act(&mut g, Action::Pass);
    let negate = fixture(&mut g, p.other(), "Negate", Zone::Hand);
    g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 1];
    cast(&mut g, negate, Some(Target::Spell(spell)));
    finish(&mut g);
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    assert!(delayed(&g).is_empty());
    let spell = fixture(&mut g, p, "The Eagles Are Coming!", Zone::Hand);
    g.state.players[p.index()].mana = mana(true);
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, Action::ChooseKicker { pay: true });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    act(&mut g, Action::FinishTargets);
    g.state.players[p.index()].mana = [0; 6];
    reject(
        &mut g,
        Action::PayCosts {
            payment: mana(true),
        },
    );
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    assert!(delayed(&g).is_empty());
}
