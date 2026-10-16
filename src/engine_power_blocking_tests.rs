use super::tests::{act, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn setup(g: &mut Game, a: CardId) {
    g.state.step = Step::Blockers;
    g.state.pending = Pending::Blockers;
    g.state.attackers.push(Attacker {
        card: a,
        defender: Some(CombatDefender::Player(Player::P1)),
        blocked: false,
    });
}
fn has(g: &Game, b: CardId, a: CardId) -> bool {
    g.legal_actions().contains(&Action::Block {
        blocker: b,
        attacker: a,
    })
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
#[test]
fn permanent_and_formal_skulk_use_strict_live_signed_power_without_conflating_abilities() {
    for formal in [false, true] {
        for ap in [-(1i128 << 80), -2, 0, 1, 3, 1i128 << 80] {
            let mut g = scenario();
            let a = named(
                &mut g,
                Player::P0,
                if formal {
                    "Grizzly Bears"
                } else {
                    "Prehistoric Pet"
                },
                Zone::Battlefield,
            );
            let b = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
            if formal {
                g.state.objects[a as usize].gained_keywords = Keywords::SKULK;
            } else {
                assert!(!g.keywords(a).contains(Keywords::SKULK));
                assert!(g.keywords(a).contains(Keywords::NO_GREATER_POWER_BLOCKERS));
            }
            g.state.objects[a as usize].growth.power = ap - if formal { 2 } else { 1 };
            setup(&mut g, a);
            for bp in [-(1i128 << 90), -3, 0, 1, 2, 3, 4, 1i128 << 90] {
                g.state.objects[b as usize].growth.power = bp - 2;
                let saved = g.snapshot();
                assert_eq!(g.can_block_pair(b, a), bp <= ap);
                assert_eq!(has(&g, b, a), bp <= ap);
                assert_eq!(g.snapshot(), saved);
            }
        }
    }
}
#[test]
fn threshold_boundaries_are_exact_and_temporary_restrictions_expire() {
    for name in [
        "April O'Neil, Kunoichi Trainee",
        "Cavern Stomper",
        "Verdant Outrider",
    ] {
        let mut g = scenario();
        let a = named(&mut g, Player::P0, name, Zone::Battlefield);
        let b = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let temporary = name != "April O'Neil, Kunoichi Trainee";
        if temporary {
            assert!(g.can_block_pair(b, a));
            g.state.players[0].mana = [0, 0, 0, 0, 4, 0];
            let action = g
                .legal_actions()
                .into_iter()
                .find(|x| matches!(x,Action::Activate{card,..}if *card==a))
                .unwrap();
            replay(&mut g, action);
            assert!(g.can_block_pair(b, a));
            passes(&mut g);
            assert!(g.keywords(a).contains(Keywords::NO_POWER_LE_TWO_BLOCKERS));
        }
        setup(&mut g, a);
        for power in [-(1i128 << 80), -1, 0, 1, 2, 3, 4, 1i128 << 80] {
            g.state.objects[b as usize].growth.power = power - 2;
            let expected = if temporary { power > 2 } else { power < 3 };
            assert_eq!(g.can_block_pair(b, a), expected, "{name} {power}");
            assert_eq!(has(&g, b, a), expected);
            if !expected {
                reject(
                    &mut g,
                    Action::Block {
                        blocker: b,
                        attacker: a,
                    },
                );
            }
        }
        g.state.objects[b as usize].growth.power = 0;
        g.finish_cleanup();
        g.state.active = Player::P0; // probe the same attacker/defender roles after expiry
        assert!(g.can_block_pair(b, a));
        assert!(!g.keywords(a).contains(Keywords::NO_POWER_LE_TWO_BLOCKERS));
        if !temporary {
            assert!(g.keywords(a).contains(Keywords::NO_POWER_GE_THREE_BLOCKERS));
        }
    }
}
#[test]
fn comparisons_include_counters_equipment_and_temporary_modifiers() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Prehistoric Pet", Zone::Battlefield);
    let b = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let equipment = named(&mut g, Player::P1, "Gold Pan", Zone::Battlefield);
    assert!(!g.can_block_pair(b, a));
    g.state.objects[a as usize].counters.plus_one = 1;
    assert!(g.can_block_pair(b, a));
    g.state.objects[equipment as usize].attached_to = Some(g.object_ref(b));
    assert_eq!(g.stats(b).power, 3);
    assert!(!g.can_block_pair(b, a));
    g.state.objects[a as usize].growth.power = 1;
    assert_eq!(g.stats(a).power, 3);
    assert!(g.can_block_pair(b, a));
    g.state.objects[b as usize].growth.power = 1;
    assert!(!g.can_block_pair(b, a));
    g.move_object(equipment, Zone::Graveyard);
    assert!(g.can_block_pair(b, a));
}
#[test]
fn power_evasion_composes_with_flight_tokens_and_exact_menace_completion() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Prehistoric Pet", Zone::Battlefield);
    let b = named(&mut g, Player::P1, "Marauding Mako", Zone::Battlefield);
    let big = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[a as usize].gained_keywords = Keywords::MENACE;
    setup(&mut g, a);
    assert!(g.can_block_pair(b, a));
    assert!(!has(&g, b, a));
    assert!(!g.can_block_pair(big, a));
    let c = named(&mut g, Player::P1, "Aegis Turtle", Zone::Battlefield);
    assert!(has(&g, b, a));
    replay(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: a,
        },
    );
    reject(&mut g, Action::FinishBlockers);
    replay(
        &mut g,
        Action::Block {
            blocker: c,
            attacker: a,
        },
    );
    replay(&mut g, Action::FinishBlockers);
    assert!(g.state.attackers[0].blocked);
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Prehistoric Pet", Zone::Battlefield);
    let b = named(&mut g, Player::P1, "Aegis Turtle", Zone::Battlefield);
    g.state.objects[a as usize].gained_keywords = Keywords::FLYING;
    assert!(!g.can_block_pair(b, a));
    g.state.objects[b as usize].gained_keywords = Keywords::REACH;
    assert!(g.can_block_pair(b, a));
    g.state.objects[b as usize].growth.power = 2;
    assert!(!g.can_block_pair(b, a));
    let token = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.name == "Goblin Token"))
        .unwrap();
    let t = spawn(&mut g, Player::P1, token, Zone::Battlefield);
    g.state.objects[a as usize].gained_keywords = Keywords::NONE;
    assert!(g.can_block_pair(t, a));
    g.state.objects[a as usize].gained_keywords = Keywords::NO_TOKEN_BLOCKERS;
    assert!(!g.can_block_pair(t, a));
}
#[test]
fn power_changes_after_declaration_do_not_remove_existing_blocks() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Prehistoric Pet", Zone::Battlefield);
    let b = named(&mut g, Player::P1, "Marauding Mako", Zone::Battlefield);
    setup(&mut g, a);
    replay(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: a,
        },
    );
    replay(&mut g, Action::FinishBlockers);
    g.state.objects[b as usize].growth.power = 3;
    assert!(!g.can_block_pair(b, a));
    assert!(g.state.attackers[0].blocked);
    g.state.step = Step::CombatDamage;
    g.prepare_damage();
    g.state_based_actions();
    assert_eq!(g.object_view(a).zone, Zone::Graveyard);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    assert_eq!(g.state.players[1].life, 20);
}
#[test]
fn complete_entry_scry_bodies_are_private_owned_and_replayable() {
    for name in ["April O'Neil, Kunoichi Trainee", "Cavern Stomper"] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        g.prepare_triggers();
        passes(&mut g);
        assert_eq!(
            g.state.pending,
            Pending::EffectLibrary { player: Player::P0 }
        );
        assert!(g.observe(Player::P1).library_choice.is_none());
        let original = g.state.players[0].library.clone();
        let a = original[original.len() - 1];
        let b = original[original.len() - 2];
        replay(
            &mut g,
            Action::ArrangeLibrary {
                card: a,
                destination: LibraryDestination::Bottom,
            },
        );
        replay(
            &mut g,
            Action::ArrangeLibrary {
                card: b,
                destination: LibraryDestination::Top,
            },
        );
        assert!(g.state.resolving.is_none());
        assert_eq!(g.state.players[0].library[0], a);
        assert_eq!(g.state.players[0].library.last(), Some(&b));
    }
}
#[test]
fn prehistoric_pet_uses_own_turn_tap_and_exact_other_creature_targets() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Prehistoric Pet", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let land = named(&mut g, Player::P0, "Plains", Zone::Battlefield);
    g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    g.state.objects[source as usize].summoning_sick = true;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==source))
    );
    g.state.objects[source as usize].summoning_sick = false;
    g.state.active = Player::P1;
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    g.state.active = Player::P0;
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for bad in [source, enemy, land] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bad),
            },
        );
    }
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    replay(
        &mut g,
        Action::PayCosts {
            payment: [2, 0, 0, 0, 0, 0],
        },
    );
    assert!(g.object_view(source).tapped);
    passes(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Hand);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
}
