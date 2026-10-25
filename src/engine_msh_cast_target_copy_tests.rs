//! Paid, bounded gated-card fixtures for cast-target, delayed-copy and retarget rules.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
const FIN: &str = "Fin Fang Foom";
const LOKI: &str = "Loki Laufeyson";
const SPEED: &str = "Speedball, New Warrior";
fn hand(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return named(g, p, name, Zone::Hand);
    }
    assert!([FIN, LOKI, SPEED].contains(&name));
    // Only these immutable typed gated definitions are seeded. Deck admission
    // remains false; no effect, target or program is replaced by the fixture.
    let id = named(g, p, "Hill Giant", Zone::Hand);
    g.state.objects[id as usize].card = card;
    id
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn packet(g: &mut Game, p: Player, name: &str, mode: Option<u16>, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = hand(g, p, name);
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic;
    if name == SPEED {
        checked(g, Action::AnnounceCast { card: id });
        let choice = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
            .unwrap();
        checked(g, choice);
        while g.state.announcing.is_some() {
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(g, pay);
        }
    } else {
        cast(g, id, mode, targets);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = packet(g, p, name, None, &[]);
    settle(g);
    id
}
fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..200 {
        if done(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("continuation did not complete");
}
fn age(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| g.state.active == p && g.state.step == Step::Main1);
    priority(g, p);
}
fn activate(g: &mut Game, id: CardId, ability: u16, mana: Mana) {
    let p = g.state.objects[id as usize].controller;
    priority(g, p);
    g.state.players[p.index()].mana = mana;
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn reject(g: &mut Game, a: Action) {
    let snapshot = g.snapshot();
    let bytes = g.state_bytes();
    let views = serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        serde_json::to_value((g.observe(Player::P0), g.observe(Player::P1))).unwrap(),
        views
    );
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut o = g.observe(p);
        o.revision = 0;
        serde_json::to_value(o).unwrap()
    })
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let before = g.snapshot();
    let before_views = views(&g);
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, rev, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn retarget(g: &mut Game) {
    until(g, |g| matches!(g.state.pending, Pending::Retarget { .. }));
}
fn counters(g: &Game, id: CardId) -> u64 {
    g.object_view(id).counters.plus_one
}
fn delayed(g: &Game) -> usize {
    g.delayed_spell_copy_view().len()
}
#[test]
fn msh_cast_copy_core_fin_paid_artifact_modal_copy_optional_targets_and_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let a = paid(&mut g, p, "Arc Reactor");
        let b = paid(&mut g, p, "Fishing Pole");
        let spell = packet(&mut g, p, "Abrade", Some(1), &[Target::Card(a)]);
        let original = g
            .state
            .stack
            .iter()
            .find(|s| s.card == spell && s.kind == StackKind::Spell)
            .unwrap()
            .clone();
        retarget(&mut g);
        assert_eq!(g.retarget_view().unwrap().item.mode, Some(1));
        assert_eq!(g.retarget_view().unwrap().item.controller, p);
        assert_eq!(
            g.retarget_view().unwrap().item.source_card,
            original.source_card
        );
        g = orphan(
            g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(counters(&g, fin), 2);
    }
}
#[test]
fn msh_cast_copy_core_fin_current_land_type_qualifies_after_paid_animation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let hall = named(&mut g, p, "Great Hall of the Biblioplex", Zone::Battlefield);
        activate(&mut g, hall, 2, [0, 0, 0, 0, 0, 5]);
        settle(&mut g);
        assert!(g.is_creature(hall));
        assert_eq!(g.object_view(hall).kinds & 16, 0);
        packet(&mut g, p, "Shore Up", None, &[Target::Card(hall)]);
        retarget(&mut g);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(counters(&g, fin), 2);
        assert_eq!(g.stats(hall).toughness, 6);
    }
}
#[test]
fn msh_cast_copy_core_fin_only_your_instant_sorcery_with_current_qualifying_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let bear = paid(&mut g, p, "Hill Giant");
        let mascot = paid(&mut g, p, "Three Tree Mascot");
        packet(&mut g, p, "Shock", None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(counters(&g, fin), 0);
        packet(&mut g, p.other(), "Shock", None, &[Target::Card(mascot)]);
        settle(&mut g);
        assert_eq!(counters(&g, fin), 0);
        let pole = paid(&mut g, p, "Fishing Pole");
        let source = g.object_view(fin);
        let bound = g.bind_target(Target::Card(pole));
        g.move_object(pole, Zone::Graveyard);
        assert!(!g.cast_filter_matches_value(
            Some(p),
            crate::cards::CastFilter::Targeted {
                filter: &crate::cards::CastFilter::InstantSorcery,
                target: crate::cards::CastTarget::PermanentKinds(18)
            },
            Card::from_name("Shock").unwrap(),
            1,
            p,
            &source,
            &[bound]
        ));
    }
}
#[test]
fn msh_cast_copy_core_loki_arms_dynamic_power_and_keeps_nonqualifying_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        age(&mut g, p);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        assert_eq!(delayed(&g), 1);
        let before = g.hand_count(p);
        packet(&mut g, p, "Quick Study", None, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 2);
        assert_eq!(delayed(&g), 1);
        activate(&mut g, loki, 1, [0, 0, 0, 1, 0, 4]);
        settle(&mut g);
        assert_eq!(g.stats(loki).power, 4);
        let before = g.hand_count(p);
        packet(&mut g, p, "Quick Study", None, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 4);
        assert_eq!(delayed(&g), 0);
    }
}
#[test]
fn msh_cast_copy_core_loki_latest_departure_power_does_not_rebind_reentered_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        age(&mut g, p);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        activate(&mut g, loki, 1, [0, 0, 0, 1, 0, 4]);
        settle(&mut g);
        packet(&mut g, p.other(), "Murder", None, &[Target::Card(loki)]);
        settle(&mut g);
        assert_eq!(g.delayed_spell_copy_view()[0].source_facts.power, 4);
        // A bounded physical return creates a new incarnation; the delayed
        // callback must continue referring to the departed power-four object.
        g.move_object(loki, Zone::Hand);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        cast(&mut g, loki, None, &[]);
        settle(&mut g);
        assert_eq!(g.stats(loki).power, 2);
        let before = g.hand_count(p);
        packet(&mut g, p, "Quick Study", None, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 4);
        assert_eq!(delayed(&g), 0);
    }
}
#[test]
fn msh_cast_copy_core_loki_source_leaves_before_registration_resolves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        age(&mut g, p);
        activate(&mut g, loki, 1, [0, 0, 0, 1, 0, 4]);
        settle(&mut g);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        packet(&mut g, p.other(), "Murder", None, &[Target::Card(loki)]);
        settle(&mut g);
        assert_eq!(delayed(&g), 1);
        assert_eq!(g.delayed_spell_copy_view()[0].source_facts.power, 4);
        let before = g.hand_count(p);
        packet(&mut g, p, "Quick Study", None, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 4);
    }
}
#[test]
fn msh_cast_copy_core_loki_multiple_registrations_consume_same_cast_and_expire() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        let echo = paid(&mut g, p, "Echo, Perceptive Prodigy");
        age(&mut g, p);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        let original = g.state.stack.last().unwrap().id;
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(original),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(delayed(&g), 2);
        let before = g.state.players[p.other().index()].life;
        packet(&mut g, p, "Shock", None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - 6);
        assert_eq!(delayed(&g), 0);
        age(&mut g, p);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        assert_eq!(delayed(&g), 1);
        next_turn(&mut g);
        settle(&mut g);
        assert_eq!(delayed(&g), 0);
    }
}
#[test]
fn msh_cast_copy_core_speedball_opponent_spell_retarget_preserves_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = packet(&mut g, p.other(), "Shock", None, &[Target::Card(speed)]);
        let original = g
            .state
            .stack
            .iter()
            .find(|s| s.card == spell && s.kind == StackKind::Spell)
            .unwrap()
            .clone();
        let count = g.state.objects.len();
        retarget(&mut g);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.stats(speed).power, 4);
        assert_eq!(g.retarget_view().unwrap().victim, Some(original.id));
        assert_eq!(g.retarget_view().unwrap().item.controller, p.other());
        g = orphan(
            g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(bear),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        assert_eq!(g.state.objects.len(), count);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.object_view(speed).damage, 0);
    }
}
#[test]
fn msh_cast_copy_core_speedball_optional_no_change_survives_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        packet(&mut g, p, "Shock", None, &[Target::Card(speed)]);
        retarget(&mut g);
        assert!(g.legal_actions().contains(&Action::FinishStackTargets));
        g = orphan(g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(
            g.stats(speed),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert_eq!(g.object_view(speed).damage, 2);
        next_turn(&mut g);
        settle(&mut g);
        assert_eq!(
            g.stats(speed),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert_eq!(g.object_view(speed).damage, 0);
    }
}
#[test]
fn msh_cast_copy_core_speedball_countered_original_still_boosts_no_stale_retarget() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        let shock = packet(&mut g, p, "Shock", None, &[Target::Card(speed)]);
        let bound = Target::Spell(shock);
        packet(&mut g, p.other(), "Negate", None, &[bound]);
        settle(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        assert_eq!(g.stats(speed).power, 4);
        assert!(g.retarget_view().is_none());
        assert_eq!(g.object_view(speed).damage, 0);
    }
}
#[test]
fn msh_cast_copy_core_speedball_modal_retarget_keeps_mode_and_rejects_illegal_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        let fin = paid(&mut g, p, FIN);
        let pole = paid(&mut g, p, "Fishing Pole");
        packet(&mut g, p.other(), "Abrade", Some(0), &[Target::Card(speed)]);
        retarget(&mut g);
        assert_eq!(g.retarget_view().unwrap().item.mode, Some(0));
        reject(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(pole),
            },
        );
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(fin),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(fin).zone, Zone::Battlefield);
        assert_eq!(g.object_view(fin).damage, 3);
        assert_eq!(counters(&g, fin), 0);
    }
}
#[test]
fn msh_cast_copy_core_bolt_bend_still_requires_changed_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let shock = packet(&mut g, p.other(), "Shock", None, &[Target::Card(a)]);
        packet(&mut g, p, "Bolt Bend", None, &[Target::Spell(shock)]);
        retarget(&mut g);
        reject(&mut g, Action::FinishStackTargets);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(a).damage, 0);
        assert_eq!(g.object_view(b).damage, 2);
    }
}
