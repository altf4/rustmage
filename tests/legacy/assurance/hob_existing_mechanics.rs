//! Oracle-derived outcomes for the HOB cards reusing established mechanics.
use super::*;

fn activate_land(g: &mut Game, land: CardId, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: land,
            ability: 2,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn tribal_lands_enter_tapped_offer_both_colors_and_pay_sacrifices_for_subtype_targets() {
    for (name, colors, recipients) in [
        ("Elvenking's Halls", [4, 1], &["Llanowar Elves"][..]),
        (
            "Goblin-town",
            [2, 3],
            &["Goblin Piker", "Plundering Pirate"][..],
        ),
        ("Iron Hills", [3, 0], &["Axgard Cavalry"][..]),
        ("Lake-town", [0, 1], &["Mild-Mannered Librarian"][..]),
        (
            "Mirkwood",
            [2, 4],
            &["Grizzly Bears", "Mineshaft Spider", "Ravening Warg"][..],
        ),
    ] {
        for &recipient in recipients {
            for p in [Player::P0, Player::P1] {
                let mut g = scenario(p);
                let land = put(&mut g, p, name, Zone::Hand);
                act(&mut g, Action::PlayLand { card: land });
                assert!(g.state.objects[land as usize].tapped, "{name}");
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::ActivateMana{card,..} if *card==land))
                );
                for color in colors {
                    g.state.objects[land as usize].tapped = false;
                    let mut production = [0; 6];
                    production[color] = 1;
                    let before = g.state.players[p.index()].mana;
                    choose(
                        &mut g,
                        |a| matches!(a, Action::ActivateMana{card,production:m,..} if *card==land && *m==production),
                    );
                    assert!(g.state.objects[land as usize].tapped);
                    assert_eq!(g.state.players[p.index()].mana[color], before[color] + 1);
                }
                g.state.objects[land as usize].tapped = false;
                let target = put(&mut g, p, recipient, Zone::Battlefield);
                let opponent = put(&mut g, p.other(), recipient, Zone::Battlefield);
                let wrong = put(&mut g, p, "Healer's Hawk", Zone::Battlefield);
                let mut funds = [0; 6];
                for color in colors {
                    funds[color] = 1;
                }
                funds[5] = 2;
                g.state.players[p.index()].mana = funds;
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: land,
                        ability: 2,
                        counter_payment: Counters::default(),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(opponent),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(wrong),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                pay(&mut g);
                assert_eq!(
                    g.state.objects[land as usize].zone,
                    Zone::Graveyard,
                    "{name}"
                );
                assert_eq!(g.state.players[p.index()].mana, [0; 6], "{name} costs");
                settle(&mut g);
                assert_eq!(
                    g.counter_count(target, CounterKind::PlusOne),
                    2,
                    "{name}/{recipient}"
                );
                assert_eq!(g.counter_count(opponent, CounterKind::PlusOne), 0);
            }
        }
    }
}

#[test]
fn tribal_land_sorcery_timing_and_departed_target() {
    let mut g = scenario(Player::P0);
    let land = put(&mut g, Player::P0, "Elvenking's Halls", Zone::Battlefield);
    let elf = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let announce = Action::AnnounceActivate {
        card: land,
        ability: 2,
        counter_payment: Counters::default(),
    };
    g.state.step = Step::BeginCombat;
    reject(&mut g, announce.clone());
    g.state.step = Step::Main1;
    g.state.active = Player::P1;
    reject(&mut g, announce);
    g.state.active = Player::P0;
    activate_land(&mut g, land, elf);
    g.move_object(elf, Zone::Hand);
    g.move_object(elf, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.counter_count(elf, CounterKind::PlusOne), 0);
    assert_eq!(g.state.objects[land as usize].zone, Zone::Graveyard);
}

#[test]
fn wood_elves_searches_nonbasic_forests_untapped_and_can_fail_to_find() {
    for p in [Player::P0, Player::P1] {
        for find in [true, false] {
            let mut g = scenario(p);
            let forest = put(&mut g, p, "Temple Garden", Zone::Library);
            let spell = put(&mut g, p, "Wood Elves", Zone::Hand);
            cast(&mut g, spell, None);
            for _ in 0..12 {
                if g.legal_actions()
                    .contains(&Action::ChooseSearch { card: forest })
                {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert!(g.observe(p.other()).search_choice.is_none());
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != forest));
            let rng_before = g.state.rng;
            if find {
                act(&mut g, Action::ChooseSearch { card: forest });
            }
            if g.legal_actions().contains(&Action::FinishSearch) {
                act(&mut g, Action::FinishSearch);
            }
            // Temple Garden's independent life replacement may request payment.
            for _ in 0..8 {
                if g.state.pending == Pending::Priority {
                    break;
                }
                act(&mut g, Action::ChooseEntryLife { pay: true });
            }
            settle(&mut g);
            assert_ne!(
                g.state.rng, rng_before,
                "search must shuffle even after failing to find"
            );
            assert_eq!(
                g.state.objects[forest as usize].zone,
                if find {
                    Zone::Battlefield
                } else {
                    Zone::Library
                }
            );
            if find {
                assert!(!g.state.objects[forest as usize].tapped);
            }
            assert_eq!(stats(&g, spell), (1, 1));
        }
    }
}

fn attack(g: &mut Game, source: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: source });
    act(g, Action::FinishAttackers);
}

#[test]
fn ferocious_attackers_check_at_attack_time_and_resolve_their_distinct_payoffs() {
    for p in [Player::P0, Player::P1] {
        for (name, base, keyword) in [
            ("Nighthowl Pursuer", (1, 1), Keywords::MENACE),
            ("Ravening Warg", (2, 2), Keywords::DEATHTOUCH),
            ("Wargling", (2, 2), Keywords::NONE),
            ("Wilderland Scrounger", (3, 6), Keywords::NONE),
        ] {
            for enabled in [false, true] {
                let mut g = scenario(p);
                let source = put(&mut g, p, name, Zone::Battlefield);
                let friend = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
                assert_eq!(stats(&g, source), base);
                if keyword != Keywords::NONE {
                    assert!(g.keywords(source).contains(keyword));
                }
                let enabler = if enabled {
                    Some(put(&mut g, p, "Serra Angel", Zone::Battlefield))
                } else {
                    None
                };
                attack(&mut g, source);
                // Losing ferocious after the trigger fires must not suppress it.
                if let Some(id) = enabler {
                    g.move_object(id, Zone::Graveyard);
                }
                settle(&mut g);
                let counter = u64::from(enabled && name == "Wilderland Scrounger");
                assert_eq!(
                    g.counter_count(source, CounterKind::PlusOne),
                    counter,
                    "{name}"
                );
                assert_eq!(
                    g.counter_count(friend, CounterKind::PlusOne),
                    counter,
                    "{name}"
                );
                assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
                assert_eq!(
                    g.state.players[p.index()].life,
                    20 + if enabled && name == "Ravening Warg" {
                        2
                    } else {
                        0
                    }
                );
                let expected = if enabled {
                    match name {
                        "Nighthowl Pursuer" => (3, 3),
                        "Wargling" => (3, 2),
                        "Wilderland Scrounger" => (4, 7),
                        _ => (2, 2),
                    }
                } else {
                    base
                };
                assert_eq!(stats(&g, source), expected, "{name}");
                assert_eq!(
                    g.keywords(friend).contains(Keywords::TRAMPLE),
                    enabled && name == "Wargling"
                );
                assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
                g.finish_cleanup();
                assert_eq!(
                    stats(&g, source),
                    (base.0 + i128::from(counter), base.1 + i128::from(counter)),
                    "{name}: cleanup"
                );
                assert!(!g.keywords(friend).contains(Keywords::TRAMPLE));
            }
        }
    }
}

#[test]
fn eagle_attack_counts_other_controlled_creatures_at_resolution_and_locks_in_bonus() {
    let mut g = scenario(Player::P0);
    let eagle = put(
        &mut g,
        Player::P0,
        "Eagle of the Great Shelf",
        Zone::Battlefield,
    );
    let friend = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&g, eagle), (2, 5));
    assert!(g.keywords(eagle).contains(Keywords::FLYING));
    attack(&mut g, eagle);
    put(&mut g, Player::P0, "Healer's Hawk", Zone::Battlefield);
    settle(&mut g);
    assert_eq!(stats(&g, eagle), (4, 7));
    g.move_object(friend, Zone::Graveyard);
    assert_eq!(stats(&g, eagle), (4, 7));
    g.finish_cleanup();
    assert_eq!(stats(&g, eagle), (2, 5));
}
