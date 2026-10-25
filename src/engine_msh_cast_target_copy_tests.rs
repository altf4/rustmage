//! Paid, bounded gated-card fixtures for cast-target, delayed-copy and retarget rules.
use super::ability_world::CharacteristicWorld;
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
fn finish_costs(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::ChooseActivationCost { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unhandled costs {:?}", g.state.pending));
        checked(g, a);
    }
}
fn xpacket(g: &mut Game, p: Player, name: &str, x: u32, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (colored, generic) = g.state.objects[id as usize].card.cost();
    g.state.players[p.index()].mana = colored;
    g.state.players[p.index()].mana[5] += generic + x;
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseX { value: x });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish_costs(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn eaten(g: &mut Game, p: Player, target: CardId, sacrifice: CardId) -> CardId {
    priority(g, p);
    let id = named(g, p, "Eaten Alive", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
    checked(g, Action::AnnounceCast { card: id });
    let mut chose = false;
    while g.state.announcing.is_some() {
        let actions = g.legal_actions();
        let a = if !chose
            && actions.contains(&Action::ChooseTarget {
                target: Target::Card(target),
            }) {
            chose = true;
            Action::ChooseTarget {
                target: Target::Card(target),
            }
        } else if actions.contains(&Action::ChooseAdditionalCost {
            object: g.object_ref(sacrifice),
        }) {
            Action::ChooseAdditionalCost {
                object: g.object_ref(sacrifice),
            }
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::ChooseSpellCost { .. }
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                    )
                })
                .unwrap_or_else(|| panic!("Eaten Alive {:?}", g.state.pending))
        };
        checked(g, a);
    }
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(sacrifice).zone, Zone::Graveyard);
    id
}
#[test]
fn msh_cast_copy_cards_printed_bodies_costs_hybrid_colors_and_sorcery_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for (name, power, toughness, types) in [
            (FIN, 3, 5, vec!["Alien", "Dragon", "Villain"]),
            (LOKI, 2, 1, vec!["God", "Sorcerer", "Villain"]),
            (SPEED, 2, 2, vec!["Human", "Hero"]),
        ] {
            let id = paid(&mut g, p, name);
            assert_eq!(g.stats(id), Stats { power, toughness });
            assert!(g.object_view(id).is_legendary());
            for ty in types {
                assert!(g.has_subtype(id, ty));
            }
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        for name in [FIN, LOKI, SPEED] {
            let mut other_turn = setup(p);
            let foreign = hand(&mut other_turn, p.other(), name);
            priority(&mut other_turn, p.other());
            assert_eq!(other_turn.state.active, p);
            assert_eq!(other_turn.state.step, Step::Main1);
            other_turn.state.players[p.other().index()].mana = [10; 6];
            reject(&mut other_turn, Action::AnnounceCast { card: foreign });
        }
        let mut g = setup(p);
        let id = hand(&mut g, p, SPEED);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: id });
        finish_costs(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(id).colors, 2 | 8);
    }
}
#[test]
fn msh_cast_copy_cards_fin_flying_rejects_ground_blocker_and_deals_combat_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        age(&mut g, p);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: fin });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: fin,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.life(p.other()), 17);
        assert!(g.object_view(fin).tapped);
    }
}
#[test]
fn msh_cast_copy_cards_fin_multigroup_duplicate_artifact_target_triggers_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let a = paid(&mut g, p, "Barkform Harvester");
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        packet(
            &mut g,
            p,
            "Epic Fight",
            Some(2),
            &[Target::Card(a), Target::Card(a), Target::Card(b)],
        );
        retarget(&mut g);
        assert_eq!(g.retarget_view().unwrap().item.targets.len(), 3);
        assert_eq!(g.retarget_view().unwrap().item.mode, Some(2));
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(counters(&g, fin), 2);
        assert_eq!(g.stats(a).power, 8);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_cast_copy_cards_fin_copy_survives_countered_original_and_adds_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let pole = paid(&mut g, p, "Fishing Pole");
        let original = packet(&mut g, p, "Abrade", Some(1), &[Target::Card(pole)]);
        packet(
            &mut g,
            p.other(),
            "Negate",
            None,
            &[Target::Spell(original)],
        );
        retarget(&mut g);
        assert_eq!(g.object_view(original).zone, Zone::Graveyard);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(pole).zone, Zone::Graveyard);
        assert_eq!(counters(&g, fin), 2);
    }
}
#[test]
fn msh_cast_copy_cards_fin_actual_undying_return_never_puts_old_counters_on_new_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let old = g.object_ref(fin);
        let pole = paid(&mut g, p, "Fishing Pole");
        packet(&mut g, p, "Abrade", Some(1), &[Target::Card(pole)]);
        packet(&mut g, p, "Undying Malice", None, &[Target::Card(fin)]);
        until(&mut g, |g| {
            g.state.resolving.is_none()
                && g.state.pending == Pending::Priority
                && g.state
                    .temporary_abilities
                    .as_ref()
                    .is_some_and(|v| v.iter().any(|r| r.recipient == old))
        });
        packet(&mut g, p.other(), "Murder", None, &[Target::Card(fin)]);
        retarget(&mut g);
        assert_ne!(g.object_ref(fin), old);
        assert_eq!(counters(&g, fin), 1);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(counters(&g, fin), 1);
        assert!(g.object_view(fin).tapped);
        assert_eq!(g.object_view(pole).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_cast_copy_cards_fin_actual_target_sacrificed_as_cost_is_ignored_without_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let mascot = paid(&mut g, p, "Three Tree Mascot");
        eaten(&mut g, p, mascot, mascot);
        assert!(!g.state.stack.iter().any(|s| s.card == fin));
        assert!(g.state.pending_triggers.iter().all(|s| s.card != fin));
        settle(&mut g);
        assert_eq!(counters(&g, fin), 0);
        assert_eq!(g.object_view(mascot).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_cast_copy_cards_fin_copies_paid_additional_sacrifice_without_repaying_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let a = paid(&mut g, p, "Three Tree Mascot");
        let b = paid(&mut g, p, "Barkform Harvester");
        let bear = paid(&mut g, p, "Bear Cub");
        eaten(&mut g, p, a, bear);
        retarget(&mut g);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(counters(&g, fin), 2);
    }
}
#[test]
fn msh_cast_copy_cards_fin_preserves_declared_counter_division_and_target_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let a = paid(&mut g, p, "Three Tree Mascot");
        let b = paid(&mut g, p, "Barkform Harvester");
        let spell = named(&mut g, p, "Biogenic Upgrade", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 3, 3];
        checked(&mut g, Action::AnnounceCast { card: spell });
        for id in [a, b] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(&mut g, Action::FinishTargets);
        checked(&mut g, Action::AllocateTargetAmount { amount: 2 });
        checked(&mut g, Action::AllocateTargetAmount { amount: 1 });
        finish_costs(&mut g);
        retarget(&mut g);
        let shares = g.retarget_view().unwrap().item.target_allocations.unwrap();
        assert_eq!(
            shares.iter().map(|r| r.amount).collect::<Vec<_>>(),
            vec![2, 1]
        );
        reject(&mut g, Action::AllocateTargetAmount { amount: 1 });
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!((counters(&g, a), counters(&g, b)), (12, 6));
        assert_eq!(counters(&g, fin), 2);
    }
}
#[test]
fn msh_cast_copy_cards_fin_preserves_chosen_x_and_does_not_count_copy_as_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fin = paid(&mut g, p, FIN);
        let a = paid(&mut g, p, "Barkform Harvester");
        let b = paid(&mut g, p, "Barkform Harvester");
        let before = g.state.players[p.index()].spells_cast_this_turn;
        xpacket(&mut g, p, "Choco-Comet", 2, &[Target::Card(a)]);
        retarget(&mut g);
        assert_eq!(g.retarget_view().unwrap().item.x, Some(2));
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(a).damage, 2);
        assert_eq!(g.object_view(b).damage, 2);
        assert_eq!(counters(&g, fin), 2);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, before + 1);
    }
}
#[test]
fn msh_cast_copy_cards_loki_real_tap_mana_sickness_powerup_discount_once_and_repeatable_arming() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: loki,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        activate(&mut g, loki, 1, [0, 0, 0, 0, 0, 3]);
        settle(&mut g);
        assert_eq!(counters(&g, loki), 2);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: loki,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        age(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: loki,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let land = named(&mut g, p, "Mountain", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: loki,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(&mut g, Action::TapMana { card: land });
        finish_costs(&mut g);
        settle(&mut g);
        assert!(g.object_view(loki).tapped);
        assert!(g.object_view(land).tapped);
        assert_eq!(delayed(&g), 1);
        packet(&mut g, p, "Shore Up", None, &[Target::Card(loki)]);
        retarget(&mut g);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        assert_eq!(delayed(&g), 1);
    }
}
#[test]
fn msh_cast_copy_cards_loki_lower_power_and_suppression_do_not_revoke_delayed_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        age(&mut g, p);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        packet(&mut g, p, "Witness Protection", None, &[Target::Card(loki)]);
        settle(&mut g);
        assert_eq!(g.stats(loki).power, 1);
        let before = g.hand_count(p);
        packet(&mut g, p, "Quick Study", None, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 2);
        assert_eq!(delayed(&g), 1);
        let life = g.life(p.other());
        packet(&mut g, p, "Shock", None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.life(p.other()), life - 4);
        assert_eq!(delayed(&g), 0);
    }
}
#[test]
fn msh_cast_copy_cards_loki_delayed_controller_is_independent_of_current_source_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        age(&mut g, p);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other()
                && g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
        });
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        packet(&mut g, p.other(), "Confiscate", None, &[Target::Card(loki)]);
        settle(&mut g);
        assert_eq!(g.object_view(loki).controller, p.other());
        let life = g.life(p.other());
        packet(&mut g, p, "Shock", None, &[Target::Player(p.other())]);
        retarget(&mut g);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.retarget_view().unwrap().item.controller, p);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.life(p.other()), life - 4);
    }
}
#[test]
fn msh_cast_copy_cards_loki_negative_departure_power_is_retained_and_nonqualifying() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        age(&mut g, p);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        packet(&mut g, p, "Witness Protection", None, &[Target::Card(loki)]);
        settle(&mut g);
        packet(&mut g, p, "Dead Weight", None, &[Target::Card(loki)]);
        settle(&mut g);
        assert_eq!(g.object_view(loki).zone, Zone::Graveyard);
        assert_eq!(g.delayed_spell_copy_view()[0].source_facts.power, -1);
        let life = g.life(p.other());
        packet(&mut g, p, "Shock", None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.life(p.other()), life - 2);
        assert_eq!(delayed(&g), 1);
    }
}
#[test]
fn msh_cast_copy_cards_loki_uses_stack_x_mana_value_and_preserves_x_in_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        age(&mut g, p);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        let before = g.hand_count(p);
        xpacket(&mut g, p, "Mind Spring", 1, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
        assert_eq!(delayed(&g), 1);
        activate(&mut g, loki, 1, [0, 0, 0, 1, 0, 4]);
        settle(&mut g);
        let before = g.hand_count(p);
        xpacket(&mut g, p, "Mind Spring", 2, &[]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 4);
        assert_eq!(delayed(&g), 0);
    }
}
#[test]
fn msh_cast_copy_cards_speedball_duplicate_multigroup_targets_boost_only_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        let giant = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        packet(
            &mut g,
            p,
            "Epic Fight",
            Some(2),
            &[
                Target::Card(speed),
                Target::Card(speed),
                Target::Card(giant),
            ],
        );
        retarget(&mut g);
        assert_eq!(g.stats(speed).power, 4);
        assert_eq!(g.retarget_view().unwrap().item.targets.len(), 3);
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.stats(speed).power, 8);
        assert_eq!(g.object_view(giant).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_cast_copy_cards_speedball_can_retarget_aura_spell_without_moving_its_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        let giant = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let aura = packet(
            &mut g,
            p,
            "Witness Protection",
            None,
            &[Target::Card(speed)],
        );
        let object = g.object_ref(aura);
        let count = g.state.objects.len();
        retarget(&mut g);
        assert_eq!(g.stats(speed).power, 4);
        g = orphan(
            g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(giant),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
        assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(giant)));
        assert_eq!(g.state.objects.len(), count);
        assert_ne!(g.object_ref(aura), object);
        assert_eq!(g.object_view(giant).name(), "Legitimate Businessperson");
        assert_eq!(g.stats(speed).power, 4);
    }
}
#[test]
fn msh_cast_copy_cards_speedball_departed_source_still_retargets_earlier_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        packet(&mut g, p.other(), "Shock", None, &[Target::Card(speed)]);
        packet(&mut g, p, "Unsummon", None, &[Target::Card(speed)]);
        retarget(&mut g);
        checked(&mut g, Action::FinishStackTargets);
        retarget(&mut g);
        assert_eq!(g.object_view(speed).zone, Zone::Hand);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(bear),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(speed).zone, Zone::Hand);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        checked(&mut g, Action::AnnounceCast { card: speed });
        finish_costs(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(speed).power, 2);
    }
}
#[test]
fn msh_cast_copy_cards_speedball_original_caster_controls_you_target_restrictions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        let fin = paid(&mut g, p, FIN);
        let giant = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        // The opponent casts this sorcery on their own turn; Speedball's
        // controller chooses targets for their spell without taking its control.
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p.other()
                && g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
        });
        packet(
            &mut g,
            p.other(),
            "Epic Fight",
            Some(1),
            &[Target::Card(giant), Target::Card(speed)],
        );
        retarget(&mut g);
        assert_eq!(g.actor(), Some(p));
        reject(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(fin),
            },
        );
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 1,
                target: Target::Card(fin),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(fin).damage, 3);
        assert_eq!(g.object_view(speed).damage, 0);
        assert_eq!(g.object_view(giant).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_cast_copy_cards_speedball_retarget_ward_is_paid_by_original_spell_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        let titan = named(&mut g, p, "Titania, Rugged Rumbler", Zone::Battlefield);
        let shock = packet(&mut g, p.other(), "Shock", None, &[Target::Card(speed)]);
        retarget(&mut g);
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(titan),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        assert_eq!(g.actor(), Some(p.other()));
        let id = g
            .state
            .stack
            .iter()
            .find(|s| s.card == shock && s.kind == StackKind::Spell)
            .unwrap()
            .id;
        assert_eq!(
            g.observe(p.other()).resolution_payment.unwrap().victim,
            Some(id)
        );
        g = orphan(g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(titan).damage, 0);
        assert_eq!(g.object_view(speed).damage, 0);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_cast_copy_cards_loki_ignores_creature_and_opponent_casts_before_next_qualifying_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = paid(&mut g, p, LOKI);
        age(&mut g, p);
        activate(&mut g, loki, 0, [0, 0, 0, 0, 0, 1]);
        settle(&mut g);
        paid(&mut g, p, "Bear Cub");
        assert_eq!(delayed(&g), 1);
        packet(&mut g, p.other(), "Shock", None, &[Target::Player(p)]);
        settle(&mut g);
        assert_eq!(delayed(&g), 1);
        let life = g.life(p.other());
        packet(&mut g, p, "Shock", None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.life(p.other()), life - 4);
        assert_eq!(delayed(&g), 0);
    }
}
#[test]
fn msh_cast_copy_cards_speedball_ignores_ability_targets_and_uncast_spell_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speed = paid(&mut g, p, SPEED);
        let raft = paid(&mut g, p, "Raft Security Officer");
        age(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: raft,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(speed),
            },
        );
        finish_costs(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(speed).power, 2);
        assert!(g.object_view(speed).tapped);
        priority(&mut g, p);
        let teach = named(&mut g, p, "Teach by Example", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: teach });
        finish_costs(&mut g);
        settle(&mut g);
        assert_eq!(delayed(&g), 1);
        let before = g.life(p.other());
        packet(&mut g, p, "Shock", None, &[Target::Card(speed)]);
        for _ in 0..2 {
            retarget(&mut g);
            if g.retarget_view().unwrap().victim.is_none() {
                checked(
                    &mut g,
                    Action::SetStackTarget {
                        index: 0,
                        target: Target::Player(p.other()),
                    },
                );
            }
            checked(&mut g, Action::FinishStackTargets);
        }
        settle(&mut g);
        assert_eq!(g.stats(speed).power, 4);
        assert_eq!(g.object_view(speed).damage, 2);
        assert_eq!(g.life(p.other()), before - 2);
    }
}
