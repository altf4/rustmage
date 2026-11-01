//! Fresh bounded assurance derived from the frozen Oracle bodies and pinned Java.
//! Fixtures establish only the stated battlefield; outcomes use paid rules actions.
use super::*;

fn fixture_token(g: &mut Game, p: Player, identity: &str) -> CardId {
    let recipe = Card::catalogue()
        .find(|c| c.identity() == identity)
        .expect("pinned token identity");
    g.create_tokens(p, recipe, 1, false);
    *tokens(g, p, recipe.name()).last().unwrap()
}

fn paid(g: &mut Game, p: Player, name: &str, mana: Mana, target: Option<Target>) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    act(
        g,
        Action::Cast {
            card: id,
            target,
            payment: mana,
        },
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6], "{name} payment");
    id
}
fn bind(g: &mut Game, targets: &[CardId]) {
    for &id in targets {
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
    if g.legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        act(g, Action::PayCosts { payment: [0; 6] });
    }
}
fn resolve_top(g: &mut Game) {
    act(g, Action::Pass);
    act(g, Action::Pass);
}
fn activate(g: &mut Game, p: Player, id: CardId, ability: u16, mana: Mana, target: Option<Target>) {
    g.state.players[p.index()].mana = mana;
    act(
        g,
        Action::Activate {
            card: id,
            ability,
            target,
            payment: mana,
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn tokens(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.name() == name
        })
        .map(|(i, _)| i as CardId)
        .collect()
}
fn step(g: &mut Game, s: Step) {
    g.begin_step(s);
    g.give_priority();
    g.prepare_triggers();
}

#[test]
fn auras_live_bonuses_and_resolved_first_strike() {
    for p in [Player::P0, Player::P1] {
        for (name, mana, expected, keys) in [
            (
                "Super Speed",
                [0, 0, 0, 1, 0, 0],
                (3, 2),
                Keywords::HASTE.union(Keywords::FIRST_STRIKE),
            ),
            (
                "Super Strength",
                [0, 0, 0, 0, 1, 4],
                (6, 6),
                Keywords::TRAMPLE,
            ),
            (
                "Super-Soldier Serum",
                [1, 0, 0, 0, 0, 1],
                (4, 4),
                Keywords::FIRST_STRIKE.union(Keywords::VIGILANCE),
            ),
        ] {
            let mut g = scenario(p);
            let c = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let aura = paid(&mut g, p, name, mana, Some(Target::Card(c)));
            settle(&mut g);
            assert_eq!(stats(&g, c), expected, "{name}");
            assert!(g.keywords(c).contains(keys));
            if name == "Super-Soldier Serum" {
                assert!(g.object_view(c).is_legendary());
                assert!(g.object_view(c).has_subtype("Soldier"));
            }
            g.move_object(aura, Zone::Graveyard);
            g.give_priority();
            settle(&mut g);
            assert_eq!(stats(&g, c), (2, 2));
            if name == "Super Speed" {
                assert!(g.keywords(c).contains(Keywords::FIRST_STRIKE));
                assert!(!g.keywords(c).contains(Keywords::HASTE));
            } else {
                assert!(!g.keywords(c).contains(keys));
            }
            g.finish_cleanup();
            assert!(!g.keywords(c).contains(Keywords::FIRST_STRIKE));
        }
    }
}

#[test]
fn equipment_enter_attach_untap_and_paid_equip() {
    for p in [Player::P0, Player::P1] {
        for (name, cost, bonus, equip_cost) in [
            ("Super Suit", [0, 1, 0, 0, 0, 1], (1, 2), 2),
            ("Vibranium Energy Daggers", [0, 0, 0, 0, 0, 1], (2, 2), 3),
        ] {
            let mut g = scenario(p);
            let c = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[c as usize].tapped = true;
            let e = paid(&mut g, p, name, cost, None);
            if name == "Super Suit" {
                resolve_top(&mut g);
                bind(&mut g, &[c]);
                settle(&mut g);
                assert!(!g.state.objects[c as usize].tapped);
            } else {
                settle(&mut g);
                activate(
                    &mut g,
                    p,
                    e,
                    0,
                    [0, 0, 0, 0, 0, equip_cost],
                    Some(Target::Card(c)),
                );
                settle(&mut g);
                assert!(g.keywords(e).contains(Keywords::INDESTRUCTIBLE));
            }
            assert_eq!(
                g.state.objects[e as usize].attached_to,
                Some(g.object_ref(c))
            );
            assert_eq!(stats(&g, c), (2 + bonus.0, 2 + bonus.1));
            let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            activate(
                &mut g,
                p,
                e,
                0,
                [0, 0, 0, 0, 0, equip_cost],
                Some(Target::Card(other)),
            );
            settle(&mut g);
            assert_eq!(stats(&g, c), (2, 2));
            assert_eq!(stats(&g, other), (2 + bonus.0, 2 + bonus.1));
        }
    }
}

#[test]
fn lockup_web_source_departure_and_wasp_blank_duration() {
    for p in [Player::P0, Player::P1] {
        for (name, cost, tapped) in [
            ("Super Villain Lockup", [1, 0, 0, 0, 0, 1], true),
            ("Web Up", [1, 0, 0, 0, 0, 2], false),
        ] {
            let mut g = scenario(p);
            let c = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[c as usize].tapped = tapped;
            let source = paid(&mut g, p, name, cost, None);
            resolve_top(&mut g);
            bind(&mut g, &[c]);
            settle(&mut g);
            assert_eq!(g.object_view(c).zone, Zone::Exile);
            g.move_object(source, Zone::Graveyard);
            g.give_priority();
            settle(&mut g);
            assert_eq!(g.object_view(c).zone, Zone::Battlefield);
            assert_eq!(g.state.objects[c as usize].controller, p.other());
        }
        let mut g = scenario(p);
        let c = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let wasp = paid(&mut g, p, "The Wondrous Wasp", [0, 1, 0, 0, 0, 1], None);
        resolve_top(&mut g);
        bind(&mut g, &[c]);
        settle(&mut g);
        assert!(g.state.objects[c as usize].tapped);
        assert_eq!(g.keywords(c), Keywords::NONE);
        g.change_controller(g.object_ref(wasp), p.other());
        assert_eq!(g.keywords(c), Keywords::NONE);
        g.move_object(wasp, Zone::Graveyard);
        g.give_priority();
        settle(&mut g);
        assert!(
            g.keywords(c)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
    }
}

#[test]
fn take_up_the_shield_and_guard_counter_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let c = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Take Up the Shield",
            [1, 0, 0, 0, 0, 1],
            Some(Target::Card(c)),
        );
        settle(&mut g);
        assert_eq!(stats(&g, c), (3, 3));
        assert!(
            g.keywords(c)
                .contains(Keywords::LIFELINK.union(Keywords::INDESTRUCTIBLE))
        );
        g.finish_cleanup();
        assert_eq!(stats(&g, c), (3, 3));
        assert_eq!(g.keywords(c), Keywords::NONE);
        for hero in [false, true] {
            let mut g = scenario(p);
            let c = put(
                &mut g,
                p.other(),
                if hero {
                    "White Tiger, Ava Ayala"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            let before = stats(&g, c);
            let guard = paid(&mut g, p, "Wakandan Royal Guard", [0, 0, 0, 0, 1, 4], None);
            resolve_top(&mut g);
            bind(&mut g, &[c]);
            settle(&mut g);
            assert_eq!(
                stats(&g, c),
                (
                    before.0 + if hero { 2 } else { 1 },
                    before.1 + if hero { 2 } else { 1 }
                )
            );
            assert!(g.keywords(guard).contains(Keywords::VIGILANCE));
        }

        let mut g = scenario(p);
        let guard = paid(&mut g, p, "Wakandan Royal Guard", [0, 0, 0, 0, 1, 4], None);
        resolve_top(&mut g);
        bind(&mut g, &[guard]);
        settle(&mut g);
        assert_eq!(stats(&g, guard), (5, 5));
    }
}

#[test]
fn land_mana_guards_and_surveillance_surveil() {
    for p in [Player::P0, Player::P1] {
        for name in ["Swamp", "Training Compound", "Surveillance Room"] {
            let mut g = scenario(p);
            let land = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            act(&mut g, Action::PlayLand { card: land });
            if name == "Surveillance Room" {
                resolve_top(&mut g);
                let top = *g.state.players[p.index()].library.last().unwrap();
                act(
                    &mut g,
                    Action::ArrangeLibrary {
                        card: top,
                        destination: LibraryDestination::Graveyard,
                    },
                );
                settle(&mut g);
                assert_eq!(g.object_view(top).zone, Zone::Graveyard);
            }
            if name == "Swamp" {
                act(&mut g, Action::TapMana { card: land });
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 0, 0]);
            }
            if name == "Training Compound" {
                let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==land&&production[3]==1)).unwrap();
                act(&mut g, action);
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
                g.state.objects[land as usize].tapped = false;
                g.state.turn += 1;
                g.state.entered_this_turn = None;
                assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==land&&(production[3]==1||production[4]==1))));
                put(&mut g, p, "Swamp", Zone::Battlefield);
                assert!(g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==land&&production[4]==1)));
            }
            if name == "Surveillance Room" {
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
                let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==land&&production[1]==1)).unwrap();
                act(&mut g, action);
                assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
            }
        }

        for name in [
            "Surveillance Room",
            "Training Compound",
            "Villainous Hideout",
        ] {
            let mut g = scenario(p);
            let land = put(&mut g, p, name, Zone::Battlefield);
            g.state.players[p.index()].mana = [0; 6];
            choose(&mut g, |a| {
                matches!(a, Action::ActivateMana { card, production, .. }
                if *card == land && *production == [0, 0, 0, 0, 0, 1])
            });
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        }
    }
}

#[test]
fn draw_payment_and_discount_families() {
    for p in [Player::P0, Player::P1] {
        for villain in [false, true] {
            let mut g = scenario(p);
            if villain {
                put(
                    &mut g,
                    p,
                    "Yellowjacket, Heartless Marauder",
                    Zone::Battlefield,
                );
            }
            paid(
                &mut g,
                p,
                "Visions of Villainy",
                [0, 0, 1, 0, 0, if villain { 1 } else { 2 }],
                None,
            );
            settle(&mut g);
            assert_eq!(g.hand_count(p), 2);
            assert_eq!(g.state.players[p.index()].life, 18);
        }
        for accept in [false, true] {
            let mut g = scenario(p);
            let artifact = put(&mut g, p, "Vibranium Energy Daggers", Zone::Battlefield);
            paid(&mut g, p, "Vision of Love", [0, 0, 0, 1, 0, 1], None);
            resolve_top(&mut g);
            if accept {
                let reference = g.object_ref(artifact);
                act(&mut g, Action::Sacrifice { object: reference });
            } else {
                act(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), if accept { 2 } else { 0 });
            assert_eq!(
                g.object_view(artifact).zone,
                if accept {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn truck_toss_vehicle_discount_and_any_target() {
    for p in [Player::P0, Player::P1] {
        for vehicle in [false, true] {
            let mut g = scenario(p);
            if vehicle {
                put(&mut g, p, "Dependable Quinjet", Zone::Battlefield);
            }
            paid(
                &mut g,
                p,
                "Truck Toss",
                [0, 0, 0, 2, 0, if vehicle { 0 } else { 2 }],
                Some(Target::Player(p.other())),
            );
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 16);
        }
    }
}

#[test]
fn live_counter_and_graveyard_statics() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let skrull = paid(&mut g, p, "Undercover Skrull", [0, 0, 0, 0, 1, 1], None);
        settle(&mut g);
        assert_eq!(stats(&g, skrull), (1, 1));
        let dead = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        put(&mut g, p, "Bear Cub", Zone::Graveyard);
        assert_eq!(stats(&g, skrull), (3, 3));
        assert!(g.object_view(skrull).has_subtype("Hero"));
        assert!(g.object_view(skrull).has_subtype("Squirrel"));
        g.move_object(dead, Zone::Exile);
        assert_eq!(stats(&g, skrull), (1, 1));
        assert!(!g.object_view(skrull).has_subtype("Hero"));
        g.state.objects[skrull as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==skrull&&production[2]==1)).unwrap();
        act(&mut g, a);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 0, 0]);
        let mut g = scenario(p);
        let c = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Training Regimen", [0, 0, 0, 0, 1, 3], None);
        settle(&mut g);
        assert!(!g.keywords(c).contains(Keywords::TRAMPLE));
        step(&mut g, Step::BeginCombat);
        bind(&mut g, &[c]);
        settle(&mut g);
        assert_eq!(stats(&g, c), (3, 3));
        assert!(g.keywords(c).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn prowess_twice_and_villain_entry_growth() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let thor = put(&mut g, p, "Thor Odinson", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        choose(&mut g, |a| matches!(a, Action::PutTrigger { .. }));
        settle(&mut g);
        assert_eq!(stats(&g, thor), (6, 6));
        assert!(
            g.keywords(thor)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        g.finish_cleanup();
        assert_eq!(stats(&g, thor), (4, 4));
        let mut g = scenario(p);
        let y = paid(
            &mut g,
            p,
            "Yellowjacket, Heartless Marauder",
            [0, 0, 1, 0, 0, 1],
            None,
        );
        settle(&mut g);
        assert_eq!(stats(&g, y), (1, 2));
        paid(&mut g, p, "Volcanic Villain", [0, 0, 0, 1, 0, 2], None);
        settle(&mut g);
        assert_eq!(stats(&g, y), (2, 2));
        assert!(g.keywords(y).contains(Keywords::LIFELINK));
        g.finish_cleanup();
        assert_eq!(stats(&g, y), (1, 2));
        assert!(!g.keywords(y).contains(Keywords::LIFELINK));
    }
}

#[test]
fn power_up_entered_discount_once_and_printed_results() {
    for p in [Player::P0, Player::P1] {
        for (name, cast_cost, power_cost, result, token) in [
            (
                "Ultron Drone",
                [0, 0, 0, 0, 0, 3],
                [0, 0, 0, 0, 0, 3],
                (4, 5),
                Some(("Robot Villain Token", (2, 2))),
            ),
            (
                "Viv Vision, Teen Synthezoid",
                [0, 0, 0, 0, 0, 3],
                [0, 0, 0, 0, 0, 4],
                (4, 4),
                None,
            ),
            (
                "Volcanic Villain",
                [0, 0, 0, 1, 0, 2],
                [0, 0, 0, 0, 0, 3],
                (5, 4),
                None,
            ),
            (
                "White Tiger, Ava Ayala",
                [0, 0, 0, 0, 1, 1],
                [0, 0, 0, 0, 0, 4],
                (3, 3),
                Some(("The Tiger God", (4, 4))),
            ),
        ] {
            let mut g = scenario(p);
            let id = paid(&mut g, p, name, cast_cost, None);
            settle(&mut g);
            activate(&mut g, p, id, 0, power_cost, None);
            settle(&mut g);
            assert_eq!(stats(&g, id), result, "{name}");
            if name == "Volcanic Villain" {
                assert!(g.keywords(id).contains(Keywords::HASTE));
            }
            if name == "Viv Vision, Teen Synthezoid" {
                assert!(g.keywords(id).contains(Keywords::FLYING));
            }
            g.state.players[p.index()].mana = [8; 6];
            assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceActivate{card,..}|Action::Activate{card,..}if *card==id)),"once {name}");
            if let Some((name, expected)) = token {
                let created = tokens(&g, p, name);
                assert_eq!(created.len(), 1);
                assert_eq!(stats(&g, created[0]), expected);
                if name == "Robot Villain Token" {
                    let v = g.object_view(created[0]);
                    assert_eq!(v.colors, 0);
                    assert_eq!(v.kinds, 17);
                    assert!(v.has_subtype("Robot"));
                    assert!(v.has_subtype("Villain"));
                }
                if name == "The Tiger God" {
                    let v = g.object_view(created[0]);
                    assert_eq!(v.colors, 16);
                    assert!(v.is_legendary());
                    assert!(v.has_subtype("Cat"));
                    assert!(v.has_subtype("God"));
                    assert!(g.keywords(created[0]).contains(Keywords::MAX_ONE_BLOCKER));
                    let first = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                    let second = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                    let god = created[0];
                    g.state.objects[god as usize].summoning_sick = false;
                    g.state.step = Step::Attackers;
                    g.state.pending = Pending::Attackers;
                    act(&mut g, Action::Attack { card: god });
                    act(&mut g, Action::FinishAttackers);
                    resolve_top(&mut g);
                    act(
                        &mut g,
                        Action::Block {
                            blocker: first,
                            attacker: god,
                        },
                    );
                    assert!(!g.legal_actions().contains(&Action::Block {
                        blocker: second,
                        attacker: god
                    }));
                    assert!(g.legal_actions().contains(&Action::FinishBlockers));
                }
            }
        }
        let mut g = scenario(p);
        let dead = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let l = paid(&mut g, p, "Unliving Legionnaire", [0, 0, 1, 0, 0, 3], None);
        settle(&mut g);
        activate(
            &mut g,
            p,
            l,
            0,
            [0, 0, 1, 0, 0, 2],
            Some(Target::Card(dead)),
        );
        settle(&mut g);
        assert_eq!(g.object_view(dead).zone, Zone::Hand);
        assert_eq!(stats(&g, l), (5, 4));
        let mut g = scenario(p);
        let w = paid(
            &mut g,
            p,
            "Wonder Man, Hollywood Hero",
            [0, 0, 0, 2, 0, 3],
            None,
        );
        settle(&mut g);
        for expected in [(6, 6), (8, 8)] {
            activate(&mut g, p, w, 0, [0, 0, 0, 0, 0, 2], None);
            settle(&mut g);
            assert_eq!(stats(&g, w), expected);
        }
        g.state.players[p.index()].mana = [8; 6];
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a,Action::AnnounceActivate{card,..}|Action::Activate{card,..}if *card==w)
        ));

        let other = paid(&mut g, p, "Volcanic Villain", [0, 0, 0, 1, 0, 2], None);
        settle(&mut g);
        for expected in [(5, 4), (7, 6)] {
            activate(&mut g, p, other, 0, [0, 0, 0, 0, 0, 3], None);
            settle(&mut g);
            assert_eq!(stats(&g, other), expected);
        }
        g.state.players[p.index()].mana = [8; 6];
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceActivate{card,..}|Action::Activate{card,..}if *card==other)));
    }
}

#[test]
fn super_skrull_all_four_activated_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = paid(&mut g, p, "Super-Skrull", [0, 0, 3, 0, 0, 1], None);
        settle(&mut g);
        assert_eq!(stats(&g, id), (4, 5));
        assert!(g.keywords(id).contains(Keywords::FLYING));
        activate(&mut g, p, id, 0, [1, 0, 0, 0, 0, 2], None);
        settle(&mut g);
        let walls = tokens(&g, p, "Wall Token");
        assert_eq!(walls.len(), 1);
        assert_eq!(stats(&g, walls[0]), (0, 4));
        assert_eq!(g.object_view(walls[0]).colors, 0);
        assert!(g.object_view(walls[0]).has_subtype("Wall"));
        assert!(g.keywords(walls[0]).contains(Keywords::DEFENDER));
        activate(&mut g, p, id, 1, [0, 0, 0, 0, 1, 3], None);
        settle(&mut g);
        assert_eq!(stats(&g, id), (8, 9));
        let c = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
        activate(&mut g, p, id, 2, [0, 0, 0, 1, 0, 4], Some(Target::Card(c)));
        settle(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
        assert_eq!(g.object_view(c).damage, 4, "red ability deals exactly four");
        activate(
            &mut g,
            p,
            id,
            3,
            [0, 1, 0, 0, 0, 5],
            Some(Target::Player(p.other())),
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p.other()), 4);
        g.finish_cleanup();
        assert_eq!(stats(&g, id), (4, 5));
    }
}

#[test]
fn sentry_shield_and_squirrel_token_recipes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = paid(
            &mut g,
            p,
            "The Sentry, Golden Guardian",
            [1, 0, 0, 0, 0, 3],
            None,
        );
        resolve_top(&mut g);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        bind(&mut g, &[]);
        settle(&mut g);
        assert_eq!(stats(&g, s), (5, 5));
        assert!(
            g.keywords(s).contains(
                Keywords::FLYING
                    .union(Keywords::VIGILANCE)
                    .union(Keywords::INDESTRUCTIBLE)
            )
        );
        let void = tokens(&g, p.other(), "The Void");
        assert_eq!(void.len(), 1);
        assert_eq!(stats(&g, void[0]), (5, 5));
        let v = g.object_view(void[0]);
        assert_eq!(v.colors, 4);
        assert!(v.is_legendary());
        assert!(v.has_subtype("Horror"));
        assert!(v.has_subtype("Villain"));
        assert!(
            g.keywords(void[0])
                .contains(Keywords::FLYING.union(Keywords::INDESTRUCTIBLE))
        );
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[void[0] as usize].summoning_sick = false;
        assert!(!g.legal_actions().contains(&Action::FinishAttackers));
        act(&mut g, Action::Attack { card: void[0] });
        act(&mut g, Action::FinishAttackers);
        let mut g = scenario(p);
        let agent = paid(
            &mut g,
            p,
            "U.S.Agent, John Walker",
            [1, 0, 0, 0, 0, 3],
            None,
        );
        settle(&mut g);
        let shield = tokens(&g, p, "Sturdy Shield");
        assert_eq!(shield.len(), 1);
        assert_eq!(
            g.state.objects[shield[0] as usize].attached_to,
            Some(g.object_ref(agent))
        );
        assert_eq!(stats(&g, agent), (4, 4));
        let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(
            &mut g,
            p,
            shield[0],
            0,
            [0, 0, 0, 0, 0, 2],
            Some(Target::Card(other)),
        );
        settle(&mut g);
        assert_eq!(stats(&g, other), (3, 4));
        assert_eq!(stats(&g, agent), (3, 2));
        let v = g.object_view(shield[0]);
        assert_eq!(v.colors, 0);
        assert!(v.has_subtype("Equipment"));
        let mut g = scenario(p);
        let girl = paid(
            &mut g,
            p,
            "The Unbeatable Squirrel Girl",
            [0, 0, 0, 0, 3, 1],
            None,
        );
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Squirrel Token").len(), 1);
        activate(&mut g, p, girl, 0, [0, 0, 0, 0, 3, 1], None);
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Squirrel Token").len(), 3);
        assert_eq!(stats(&g, girl), (4, 4));
    }
}

#[test]
fn ant_man_card_draw_and_variable_counter_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ant = paid(
            &mut g,
            p,
            "The Astonishing Ant-Man",
            [0, 1, 0, 0, 1, 0],
            None,
        );
        settle(&mut g);
        paid(&mut g, p, "Visions of Villainy", [0, 0, 1, 0, 0, 2], None);
        resolve_top(&mut g);
        choose(&mut g, |a| matches!(a, Action::PutTrigger { .. }));
        settle(&mut g);
        assert_eq!(stats(&g, ant), (3, 3));
        g.state.objects[ant as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ant,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(&mut g, Action::ChooseSourceCounterAmount { amount: 2 });
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 2],
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, ant), (1, 1));
        assert_eq!(tokens(&g, p, "Insect Token").len(), 2);
        for insect in tokens(&g, p, "Insect Token") {
            assert_eq!(stats(&g, insect), (1, 1));
            assert_eq!(g.object_view(insect).colors, 16);
        }
        assert!(g.state.objects[ant as usize].tapped);
    }
}

#[test]
fn thirst_artifact_or_two_discards() {
    for p in [Player::P0, Player::P1] {
        for artifact_route in [false, true] {
            let mut g = scenario(p);
            let discard = put(
                &mut g,
                p,
                if artifact_route {
                    "Vibranium Energy Daggers"
                } else {
                    "Swamp"
                },
                Zone::Hand,
            );
            paid(&mut g, p, "Thirst for Knowledge", [0, 1, 0, 0, 0, 2], None);
            resolve_top(&mut g);
            if artifact_route {
                act(&mut g, Action::Discard { card: discard });
            } else {
                act(&mut g, Action::DeclineEffectPayment);
                act(&mut g, Action::Discard { card: discard });
                choose(&mut g, |a| matches!(a, Action::Discard { .. }));
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), if artifact_route { 3 } else { 2 });
            assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn ten_rings_endstep_intervening_count_and_tigra_events() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid(&mut g, p, "The Ten Rings", [0, 0, 0, 0, 0, 8], None);
        settle(&mut g);
        for _ in 0..2 {
            put(&mut g, p, "Forest", Zone::Hand);
        }
        for _ in 0..4 {
            put(&mut g, p, "Forest", Zone::Library);
        }
        step(&mut g, Step::End);
        put(&mut g, p, "Forest", Zone::Hand);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 10);
        assert!(!g.above_hand_limit(p, 0));
        let extra = put(&mut g, p, "Forest", Zone::Hand);
        assert!(g.above_hand_limit(p, 0));
        g.move_object(extra, Zone::Graveyard);
        step(&mut g, Step::End);
        assert!(g.state.stack.is_empty());
        assert!(g.state.pending_triggers.is_empty());
        let mut g = scenario(p);
        let tigra = paid(&mut g, p, "Tigra, Feline Fury", [0, 0, 0, 0, 1, 1], None);
        settle(&mut g);
        for expected in [(3, 2), (4, 3)] {
            let food = fixture_token(&mut g, p, "@token/283fd83a9aec83a51b736926");
            activate(&mut g, p, food, 0, [0, 0, 0, 0, 0, 2], None);
            settle(&mut g);
            assert_eq!(stats(&g, tigra), expected);
        }
        assert_eq!(g.state.players[p.index()].life, 26);
    }
}

fn teamwork_cast(
    g: &mut Game,
    p: Player,
    name: &str,
    mana: Mana,
    using: Option<CardId>,
    mode: Option<u16>,
    targets: &[Target],
) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    act(g, Action::AnnounceCast { card: id });
    act(
        g,
        Action::ChooseTeamwork {
            pay: using.is_some(),
        },
    );
    if let Some(mode) = mode {
        act(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    act(g, Action::PayCosts { payment: mana });
    if let Some(c) = using {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseCastCost { .. }))
        {
            choose(g, |a| matches!(a, Action::ChooseCastCost { .. }));
        }
        act(
            g,
            Action::ChooseTeammate {
                object: g.object_ref(c),
            },
        );
        act(g, Action::FinishTeamwork);
    }
    id
}

#[test]
fn teamwork_tactics_reanimation_bite_and_counter_tax() {
    for p in [Player::P0, Player::P1] {
        for using in [false, true] {
            let mut g = scenario(p);
            let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            teamwork_cast(
                &mut g,
                p,
                "Team Tactics",
                [0, 0, 0, 1, 0, 1],
                using.then_some(helper),
                None,
                &[Target::Card(target)],
            );
            settle(&mut g);
            assert!(g.keywords(target).contains(Keywords::DOUBLE_STRIKE));
            assert_eq!(g.keywords(target).contains(Keywords::TRAMPLE), using);
            assert_eq!(g.state.objects[helper as usize].tapped, using);
            let mut g = scenario(p);
            let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
            let dead = put(
                &mut g,
                p,
                if using {
                    "The Thing, Ben Grimm"
                } else {
                    "Bear Cub"
                },
                Zone::Graveyard,
            );
            teamwork_cast(
                &mut g,
                p,
                "Too Evil to Stay Dead",
                [0, 0, 1, 0, 0, 2],
                using.then_some(helper),
                None,
                &[Target::Card(dead)],
            );
            settle(&mut g);
            assert_eq!(g.object_view(dead).zone, Zone::Battlefield);
            let mut g = scenario(p);
            let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
            let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            teamwork_cast(
                &mut g,
                p,
                "Widow's Bite",
                [0, 0, 1, 0, 0, 1],
                using.then_some(helper),
                Some(if using { 2 } else { 0 }),
                &[Target::Card(a), Target::Card(b)][..if using { 2 } else { 1 }],
            );
            settle(&mut g);
            assert!(g.keywords(a).contains(Keywords::DEATHTOUCH));
            assert_eq!(g.object_view(b).zone, Zone::Battlefield);
            assert_eq!(
                stats(&g, b),
                if using { (1, 1) } else { (3, 3) },
                "Bite changes a 3/3 by exactly -2/-2"
            );
            g.finish_cleanup();
            assert_eq!(stats(&g, b), (3, 3), "Bite expires at cleanup");
            assert!(!g.keywords(a).contains(Keywords::DEATHTOUCH));
            let mut g = scenario(p.other());
            let spell = paid(
                &mut g,
                p.other(),
                "Vibranium Energy Daggers",
                [0, 0, 0, 0, 0, 1],
                None,
            );
            act(&mut g, Action::Pass);
            let helper = put(&mut g, p, "Serra Angel", Zone::Battlefield);
            teamwork_cast(
                &mut g,
                p,
                "We Say Thee Nay!",
                [0, 1, 0, 0, 0, 1],
                using.then_some(helper),
                None,
                &[Target::Spell(spell)],
            );
            resolve_top(&mut g);
            g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, if using { 4 } else { 2 }];
            act(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, if using { 4 } else { 2 }],
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        }

        let mut g = scenario(p.other());
        let spell = paid(
            &mut g,
            p.other(),
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        act(&mut g, Action::Pass);
        teamwork_cast(
            &mut g,
            p,
            "We Say Thee Nay!",
            [0, 1, 0, 0, 0, 1],
            None,
            None,
            &[Target::Spell(spell)],
        );
        resolve_top(&mut g);
        act(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}

#[test]
fn titania_additional_cost_hybrid_and_ward_payer() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = scenario(p);
            let spare = put(&mut g, p, "Forest", Zone::Hand);
            let id = put(&mut g, p, "Titania, Rugged Rumbler", Zone::Hand);
            let payment = [0, 0, 1, 0, 0, if discard { 2 } else { 4 }];
            g.state.players[p.index()].mana = payment;
            act(&mut g, Action::AnnounceCast { card: id });
            let cost = g
                .observe(p)
                .cost_choices
                .into_iter()
                .find(|c| {
                    c.cost.colored == [0, 0, 1, 0, 0, 0]
                        && c.cost.generic == if discard { 2 } else { 4 }
                })
                .expect("printed hybrid base plus independent additional route");
            act(&mut g, Action::ChooseSpellCost { cost: cost.index });
            if discard {
                let reference = g.object_ref(spare);
                act(&mut g, Action::ChooseAdditionalCost { object: reference });
            }
            act(&mut g, Action::PayCosts { payment });
            settle(&mut g);
            assert_eq!(stats(&g, id), (5, 5));
            assert_eq!(
                g.object_view(spare).zone,
                if discard { Zone::Graveyard } else { Zone::Hand }
            );
            g.state.priority = p.other();
            let ward_discard = discard.then(|| put(&mut g, p.other(), "Forest", Zone::Hand));
            let bounce = paid(
                &mut g,
                p.other(),
                "Unsummon",
                [0, 1, 0, 0, 0, 0],
                Some(Target::Card(id)),
            );
            g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 2];
            resolve_top(&mut g);
            if let Some(card) = ward_discard {
                act(&mut g, Action::Discard { card });
            } else {
                act(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    },
                );
            }
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Hand);
            assert_eq!(g.object_view(bounce).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn thanos_parity_zero_even_and_old_source_exclusion() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let thanos = paid(&mut g, p, "Thanos, the Mad Titan", [1, 0, 1, 1, 0, 0], None);
        settle(&mut g);
        let even = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let zero = fixture_token(&mut g, p.other(), "@token/b73d837afe1978c0c8507afb");
        let odd = put(&mut g, p, "Volcanic Villain", Zone::Battlefield);
        activate(&mut g, p, thanos, 0, [0, 1, 0, 0, 1, 1], None);
        resolve_top(&mut g);
        act(
            &mut g,
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Even,
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, thanos), (6, 6));
        assert_eq!(g.object_view(even).zone, Zone::Graveyard);
        assert!(!g.on_battlefield(zero));
        assert_eq!(g.object_view(odd).zone, Zone::Battlefield);
    }
}

#[test]
fn taskmaster_copy_exceptions_and_next_turn_expiry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let donor = put(&mut g, p.other(), "Ultron Drone", Zone::Battlefield);
        let task = paid(
            &mut g,
            p,
            "Taskmaster, Mercenary Mimic",
            [0, 1, 1, 0, 0, 2],
            None,
        );
        settle(&mut g);
        g.state.turn += 2;
        g.begin_turn();
        step(&mut g, Step::Main1);
        bind(&mut g, &[donor]);
        settle(&mut g);
        let view = g.object_view(task);
        assert_eq!(view.name(), "Taskmaster, Mercenary Mimic");
        assert_eq!(stats(&g, task), (2, 3));
        assert!(view.is_legendary());
        assert_eq!(view.kinds, 1);
        assert!(view.has_subtype("Human"));
        assert!(view.has_subtype("Mercenary"));
        assert!(view.has_subtype("Villain"));
        assert!(!view.has_subtype("Robot"));
        g.move_object(donor, Zone::Graveyard);
        assert_eq!(stats(&g, task), (2, 3));
        g.state.turn += 1;
        g.state.active = p.other();
        g.begin_turn();
        assert_eq!(stats(&g, task), (2, 3));
        g.state.turn += 1;
        g.state.active = p;
        g.begin_turn();
        assert_eq!(stats(&g, task), (3, 5));
    }
}

#[test]
fn vision_modes_viv_attack_and_war_machine_source_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let vision = paid(&mut g, p, "The Vision", [0, 0, 0, 0, 0, 4], None);
        settle(&mut g);
        for mode in [0, 1, 2] {
            paid(
                &mut g,
                p,
                "Vibranium Energy Daggers",
                [0, 0, 0, 0, 0, 1],
                None,
            );
            let modes = g
                .legal_actions()
                .into_iter()
                .filter_map(|a| {
                    if let Action::ChooseMode { mode } = a {
                        Some(mode)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(modes, (mode..3).collect::<Vec<_>>());
            act(&mut g, Action::ChooseMode { mode });
            settle(&mut g);
        }
        assert!(
            g.keywords(vision)
                .contains(Keywords::DOUBLE_STRIKE.union(Keywords::INDESTRUCTIBLE))
        );
        assert_eq!(g.hand_count(p), 1);
        paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        assert_eq!(g.state.pending, Pending::Priority);
        settle(&mut g);
        g.state.turn += 2;
        g.begin_turn();
        step(&mut g, Step::Main1);
        paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        assert!(g.legal_actions().contains(&Action::ChooseMode { mode: 0 }));
        act(&mut g, Action::ChooseMode { mode: 0 });
        settle(&mut g);
        let mut g = scenario(p);
        let viv = put(&mut g, p, "Viv Vision, Teen Synthezoid", Zone::Battlefield);

        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: viv });
        act(&mut g, Action::FinishAttackers);
        g.add_counter(viv, CounterKind::PlusOne, 2);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
        let mut g = scenario(p);
        let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let war = paid(
            &mut g,
            p,
            "War Machine, Legacy of Iron",
            [1, 0, 0, 0, 0, 2],
            None,
        );
        settle(&mut g);
        step(&mut g, Step::BeginCombat);
        bind(&mut g, &[other]);
        g.add_counter(war, CounterKind::PlusOne, 2);
        settle(&mut g);
        assert_eq!(stats(&g, other), (5, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, other), (2, 2));
    }
}

#[test]
fn sagas_all_chapters_and_final_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let saga = paid(
            &mut g,
            p,
            "The Coming of Galactus",
            [0, 0, 2, 0, 1, 2],
            None,
        );
        resolve_top(&mut g);
        bind(&mut g, &[victim]);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        for life in [18, 16] {
            g.state.turn += 2;
            step(&mut g, Step::Main1);
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, life);
        }
        g.state.turn += 2;
        step(&mut g, Step::Main1);
        settle(&mut g);
        assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
        let gal = tokens(&g, p, "Galactus");
        assert_eq!(gal.len(), 1);
        assert_eq!(stats(&g, gal[0]), (16, 16));
        let v = g.object_view(gal[0]);
        assert_eq!(v.colors, 4);
        assert!(v.is_legendary());
        assert!(v.has_subtype("Elder"));
        assert!(v.has_subtype("Alien"));
        assert!(
            g.keywords(gal[0])
                .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
        );
        let land = put(&mut g, p, "Swamp", Zone::Battlefield);
        g.state.objects[gal[0] as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: gal[0] });
        act(&mut g, Action::FinishAttackers);
        bind(&mut g, &[land]);
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        let mut g = scenario(p);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let saga = paid(&mut g, p, "World War Hulk", [0, 0, 0, 0, 2, 3], None);
        settle(&mut g);
        let red = put(&mut g, p, "Volcanic Villain", Zone::Hand);
        assert!(
            g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceFree{card,..}if *card==red))
        );
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        act(
            &mut g,
            Action::Cast {
                card: red,
                target: None,
                payment: [0, 0, 0, 1, 0, 2],
            },
        );
        settle(&mut g);
        let next = put(&mut g, p, "Volcanic Villain", Zone::Hand);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceFree{card,..}if *card==next))
        );
        g.state.turn += 2;
        step(&mut g, Step::Main1);
        bind(&mut g, &[target]);
        settle(&mut g);
        assert_eq!(stats(&g, target), (5, 5));
        g.state.turn += 2;
        step(&mut g, Step::Main1);
        bind(&mut g, &[target]);
        settle(&mut g);
        assert_eq!(stats(&g, target), (10, 10));
        assert!(g.keywords(target).contains(Keywords::TRAMPLE));
        assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
        paid(
            &mut g,
            p,
            "Take Up the Shield",
            [1, 0, 0, 0, 0, 1],
            Some(Target::Card(target)),
        );
        settle(&mut g);
        assert_eq!(stats(&g, target), (11, 11));
    }
}

#[test]
fn civil_war_control_boost_fight_and_duration() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Volcanic Villain", Zone::Battlefield);
        let fighter = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let saga = paid(
            &mut g,
            p,
            "The Super Hero Civil War",
            [1, 0, 0, 1, 0, 3],
            None,
        );
        resolve_top(&mut g);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(enemy)
        }));
        bind(&mut g, &[a]);
        settle(&mut g);
        assert_eq!(g.state.objects[a as usize].controller, p);
        assert_eq!(g.state.objects[b as usize].controller, p);
        g.state.turn += 2;
        step(&mut g, Step::Main1);
        settle(&mut g);
        assert_eq!(stats(&g, fighter), (3, 3));
        assert!(g.keywords(fighter).contains(Keywords::VIGILANCE));
        g.finish_cleanup();
        g.state.active = p;
        g.state.priority = p;
        g.state.turn += 1;
        g.begin_turn();
        step(&mut g, Step::Main1);
        bind(&mut g, &[fighter, enemy]);
        settle(&mut g);
        assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
        assert_eq!(g.state.objects[a as usize].controller, p.other());
        assert_eq!(g.state.objects[b as usize].controller, p.other());
        assert_eq!(g.object_view(fighter).zone, Zone::Graveyard);
        assert_eq!(g.state.objects[enemy as usize].damage, 2);
    }
}

#[test]
fn tony_both_faces_top_four_transform_and_successful_attachment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let artifact = put(&mut g, p, "Vibranium Energy Daggers", Zone::Library);
        let remainder = (0..3)
            .map(|_| put(&mut g, p, "Forest", Zone::Library))
            .collect::<Vec<_>>();
        let unseen_top =
            g.state.players[p.index()].library[g.state.players[p.index()].library.len() - 5];
        let rng_before = g.state.rng.0;
        let tony = paid(&mut g, p, "Tony Stark", [0, 1, 0, 0, 0, 1], None);
        settle(&mut g);
        assert_eq!(stats(&g, tony), (1, 3));
        g.state.objects[tony as usize].summoning_sick = false;
        activate(&mut g, p, tony, 0, [0, 0, 0, 0, 0, 1], None);
        resolve_top(&mut g);
        act(&mut g, Action::SelectLibrary { card: artifact });
        act(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Hand);
        let library = &g.state.players[p.index()].library;
        assert_eq!(library.last(), Some(&unseen_top));
        assert!(remainder.iter().all(|id| library[..3].contains(id)));
        assert_ne!(g.state.rng.0, rng_before);
        activate(&mut g, p, tony, 1, [0, 1, 0, 1, 0, 4], None);
        settle(&mut g);
        assert_eq!(g.object_view(tony).name(), "The Invincible Iron Man");
        assert_eq!(stats(&g, tony), (5, 5));
        assert!(
            g.keywords(tony)
                .contains(Keywords::FLYING.union(Keywords::HASTE))
        );
        step(&mut g, Step::BeginCombat);
        resolve_top(&mut g);
        let entrant = g.object_ref(artifact);
        act(&mut g, Action::PutHandPermanent { object: entrant });
        settle(&mut g);
        assert_eq!(
            g.state.objects[artifact as usize].attached_to,
            Some(g.object_ref(tony))
        );
        assert_eq!(stats(&g, tony), (7, 7));
    }
}

#[test]
fn ultron_copy_animation_and_worlds_hand_exchange() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid(
            &mut g,
            p,
            "Ultron, Artificial Malevolence",
            [0, 0, 0, 0, 0, 3],
            None,
        );
        settle(&mut g);
        paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        resolve_top(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        resolve_top(&mut g);
        act(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        settle(&mut g);
        let copies = tokens(&g, p, "Vibranium Energy Daggers");
        assert_eq!(copies.len(), 1);
        assert_eq!(stats(&g, copies[0]), (2, 2));
        assert!(g.object_view(copies[0]).has_subtype("Robot"));
        assert!(g.object_view(copies[0]).has_subtype("Villain"));
        let mut g = scenario(p);
        let old = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let stolen = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.change_controller(g.object_ref(stolen), p);
        let a = put(&mut g, p, "Volcanic Villain", Zone::Hand);
        let b = put(&mut g, p.other(), "Ultron Drone", Zone::Hand);
        let spell = paid(&mut g, p, "Worlds Within Worlds", [0, 1, 0, 0, 1, 5], None);
        resolve_top(&mut g);
        let ar = g.object_ref(a);
        act(&mut g, Action::ChooseHandCard { object: Some(ar) });
        act(&mut g, Action::ChooseHandCard { object: None });
        let br = g.object_ref(b);
        act(&mut g, Action::ChooseHandCard { object: Some(br) });
        act(&mut g, Action::ChooseHandCard { object: None });
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(g.object_view(b).zone, Zone::Battlefield);
        assert_eq!(g.object_view(old).zone, Zone::Hand);
        assert_eq!(g.object_view(stolen).zone, Zone::Hand);
        assert_eq!(g.state.objects[stolen as usize].owner, p.other());
        assert_eq!(g.object_view(spell).zone, Zone::Exile);

        for accept in [false, true] {
            let mut g = scenario(p);
            paid(
                &mut g,
                p,
                "Ultron, Artificial Malevolence",
                [0, 0, 0, 0, 0, 3],
                None,
            );
            settle(&mut g);
            paid(&mut g, p, "Ultron Drone", [0, 0, 0, 0, 0, 3], None);
            resolve_top(&mut g);
            resolve_top(&mut g);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            if accept {
                act(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    },
                );
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            let copies = tokens(&g, p, "Ultron Drone");
            assert_eq!(copies.len(), usize::from(accept));
            if accept {
                assert_eq!(stats(&g, copies[0]), (2, 3));
            }
        }
    }
}

#[test]
fn vision_quest_search_and_drone_scry_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let donor = put(&mut g, p, "Ultron Drone", Zone::Graveyard);
        let spell = put(&mut g, p, "Vision Quest", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 4];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseX { value: 4 });
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 1, 0, 4],
            },
        );
        resolve_top(&mut g);
        act(
            &mut g,
            Action::ChooseSearchZones {
                library: false,
                graveyard: true,
            },
        );
        act(&mut g, Action::ChooseSearch { card: donor });
        if g.legal_actions().contains(&Action::FinishSearch) {
            act(&mut g, Action::FinishSearch);
        }
        settle(&mut g);
        assert_eq!(stats(&g, donor), (6, 7));
        assert!(g.keywords(donor).contains(Keywords::HASTE));
        let mut g = scenario(p);
        let top = put(&mut g, p, "Swamp", Zone::Library);
        let first = put(&mut g, p, "Forest", Zone::Library);
        let flock = paid(&mut g, p, "Wakandan Drone Flock", [1, 0, 0, 0, 0, 3], None);
        resolve_top(&mut g);
        resolve_top(&mut g);
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: first,
                destination: LibraryDestination::Bottom,
            },
        );
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&top));
        assert_eq!(g.state.players[p.index()].library.first(), Some(&first));
        assert_eq!(stats(&g, flock), (3, 3));
        assert!(g.keywords(flock).contains(Keywords::FLYING));

        let mut g = scenario(p);
        let donor = put(&mut g, p, "Ultron Drone", Zone::Library);
        let rng_before = g.state.rng.0;
        let spell = put(&mut g, p, "Vision Quest", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 4];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseX { value: 4 });
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 1, 0, 4],
            },
        );
        resolve_top(&mut g);
        act(
            &mut g,
            Action::ChooseSearchZones {
                library: true,
                graveyard: false,
            },
        );
        act(&mut g, Action::ChooseSearch { card: donor });
        if g.legal_actions().contains(&Action::FinishSearch) {
            act(&mut g, Action::FinishSearch);
        }
        settle(&mut g);
        assert_eq!(g.object_view(donor).zone, Zone::Battlefield);
        assert_eq!(stats(&g, donor), (6, 7));
        assert_ne!(g.state.rng.0, rng_before);
    }
}

#[test]
fn scarlet_reduction_masters_lord_and_plan_tutor() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid(&mut g, p, "The Scarlet Witch", [0, 0, 0, 1, 0, 2], None);
        settle(&mut g);
        paid(
            &mut g,
            p,
            "Truck Toss",
            [0, 0, 0, 2, 0, 0],
            Some(Target::Player(p.other())),
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        paid(&mut g, p, "Visions of Villainy", [0, 0, 1, 0, 0, 2], None);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 2);
        let mut g = scenario(p);
        let v = put(
            &mut g,
            p,
            "Yellowjacket, Heartless Marauder",
            Zone::Battlefield,
        );
        let masters = paid(&mut g, p, "The Masters of Evil", [0, 0, 1, 0, 0, 5], None);
        settle(&mut g);
        assert_eq!(stats(&g, v), (4, 3));
        assert_eq!(stats(&g, masters), (5, 6));
        let mut g = scenario(p);
        let plan = put(&mut g, p, "Robot Domination", Zone::Library);
        let masters = put(&mut g, p, "The Masters of Evil", Zone::Hand);
        activate(&mut g, p, masters, 0, [0, 0, 1, 0, 0, 1], None);
        assert_eq!(g.object_view(masters).zone, Zone::Graveyard);
        resolve_top(&mut g);
        act(&mut g, Action::ChooseSearch { card: plan });
        if g.legal_actions().contains(&Action::FinishSearch) {
            act(&mut g, Action::FinishSearch);
        }
        settle(&mut g);
        assert_eq!(g.object_view(plan).zone, Zone::Hand);
    }
}

#[test]
fn adaptoid_all_keyword_counters_and_legendary_cda() {
    let keywords = [
        (CounterKind::Haste, Keywords::HASTE),
        (CounterKind::Flying, Keywords::FLYING),
        (CounterKind::FirstStrike, Keywords::FIRST_STRIKE),
        (CounterKind::DoubleStrike, Keywords::DOUBLE_STRIKE),
        (CounterKind::Deathtouch, Keywords::DEATHTOUCH),
        (CounterKind::Indestructible, Keywords::INDESTRUCTIBLE),
        (CounterKind::Lifelink, Keywords::LIFELINK),
        (CounterKind::Menace, Keywords::MENACE),
        (CounterKind::Reach, Keywords::REACH),
        (CounterKind::Trample, Keywords::TRAMPLE),
        (CounterKind::Vigilance, Keywords::VIGILANCE),
    ];
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "White Tiger, Ava Ayala", Zone::Battlefield);
        put(&mut g, p, "The Wondrous Wasp", Zone::Battlefield);
        let donor = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        // Bounded donor fixture supplies each keyword independently of a card program.
        for &(kind, _) in &keywords {
            g.add_counter(donor, kind, 1);
        }
        let a = put(&mut g, p, "Super-Adaptoid", Zone::Hand);
        assert_eq!(stats(&g, a), (2, 2));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(
            &mut g,
            Action::Cast {
                card: a,
                target: None,
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        resolve_top(&mut g);
        bind(&mut g, &[donor]);
        settle(&mut g);
        assert_eq!(stats(&g, a), (3, 2));
        for &(kind, key) in &keywords {
            assert_eq!(g.counter_count(a, kind), 1);
            assert!(g.keywords(a).contains(key));
        }
        g.state.objects[a as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: a });
        act(&mut g, Action::FinishAttackers);
        bind(&mut g, &[donor]);
        settle(&mut g);
        for &(kind, _) in &keywords {
            assert_eq!(g.counter_count(a, kind), 1);
        }
    }
}

#[test]
fn equipment_attack_attachments_connive_and_finality() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let c = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        settle(&mut g);
        paid(
            &mut g,
            p,
            "Super-Soldier Serum",
            [1, 0, 0, 0, 0, 1],
            Some(Target::Card(c)),
        );
        settle(&mut g);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: c });
        act(&mut g, Action::FinishAttackers);
        bind(&mut g, &[e]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[e as usize].attached_to,
            Some(g.object_ref(c))
        );
        assert_eq!(stats(&g, c), (6, 6));
        let mut g = scenario(p);
        let sword = paid(
            &mut g,
            p,
            "Swordsman, Sharp Scoundrel",
            [0, 0, 1, 0, 0, 1],
            None,
        );
        settle(&mut g);
        let e = paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        settle(&mut g);
        paid(&mut g, p, "Volcanic Villain", [0, 0, 0, 1, 0, 2], None);
        resolve_top(&mut g);
        bind(&mut g, &[e, sword]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[e as usize].attached_to,
            Some(g.object_ref(sword))
        );
        let discard = put(&mut g, p, "Truck Toss", Zone::Hand);
        g.state.objects[sword as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: sword });
        act(&mut g, Action::FinishAttackers);
        resolve_top(&mut g);
        act(&mut g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(stats(&g, sword), (5, 5));
        let mut g = scenario(p);
        let e = put(&mut g, p, "Vibranium Energy Daggers", Zone::Battlefield);
        let winter = put(&mut g, p, "Winter Soldier, Icy Assassin", Zone::Graveyard);
        activate(&mut g, p, winter, 0, [1, 0, 1, 0, 0, 3], None);
        resolve_top(&mut g);
        let er = g.object_ref(e);
        act(
            &mut g,
            Action::ChooseEquipmentAttachment {
                equipment: Some(er),
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, winter), (6, 4));
        g.move_object(winter, Zone::Graveyard);
        g.give_priority();
        assert_eq!(g.object_view(winter).zone, Zone::Exile);
        let mut g = scenario(p);
        let whip = paid(
            &mut g,
            p,
            "Whiplash, Vengeful Engineer",
            [0, 0, 1, 0, 0, 0],
            None,
        );
        settle(&mut g);
        assert!(g.state.objects[whip as usize].tapped);
        let e = put(&mut g, p, "Vibranium Energy Daggers", Zone::Battlefield);
        activate(
            &mut g,
            p,
            e,
            0,
            [0, 0, 0, 0, 0, 3],
            Some(Target::Card(whip)),
        );
        settle(&mut g);
        g.state.objects[whip as usize].tapped = false;
        g.state.objects[whip as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: whip });
        act(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);

        let mut g = scenario(p);
        let defender = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let attacker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let e = put(&mut g, p, "Vibranium Energy Daggers", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Super-Soldier Serum",
            [1, 0, 0, 0, 0, 1],
            Some(Target::Card(defender)),
        );
        settle(&mut g);
        g.state.active = p.other();
        g.state.priority = p.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        resolve_top(&mut g);
        act(
            &mut g,
            Action::Block {
                blocker: defender,
                attacker,
            },
        );
        act(&mut g, Action::FinishBlockers);
        bind(&mut g, &[e]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[e as usize].attached_to,
            Some(g.object_ref(defender))
        );
    }
}

#[test]
fn blink_sources_equipment_draw_and_hero_damage_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let c = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let old = g.object_ref(c);
        let thor = paid(
            &mut g,
            p,
            "The Mighty Thor, Jane Foster",
            [1, 1, 0, 0, 0, 1],
            None,
        );
        settle(&mut g);
        g.state.objects[thor as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: thor });
        act(&mut g, Action::FinishAttackers);
        bind(&mut g, &[c]);
        settle(&mut g);
        assert_ne!(g.object_ref(c), old);
        assert_eq!(g.state.objects[c as usize].controller, p.other());
        assert!(g.state.objects[c as usize].tapped);
        g.state.step = Step::Main2;
        g.state.pending = Pending::Priority;
        g.state.priority = p;
        paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
        let mut g = scenario(p);
        let c = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let old = g.object_ref(c);
        let stone = paid(&mut g, p, "The Mind Stone", [1, 0, 0, 0, 0, 1], None);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production,..}if *card==stone&&*production==[1,0,0,0,0,0]),
        );
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        g.state.objects[stone as usize].tapped = false;
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 5];
        choose(
            &mut g,
            |a| matches!(a,Action::Activate{card,payment,..}if *card==stone&&*payment==[1,0,0,0,0,5]),
        );
        settle(&mut g);
        step(&mut g, Step::End);
        bind(&mut g, &[c]);
        settle(&mut g);
        assert_ne!(g.object_ref(c), old);
        assert!(g.keywords(stone).contains(Keywords::INDESTRUCTIBLE));
        let mut g = scenario(p);
        let thing = paid(&mut g, p, "The Thing, Ben Grimm", [0, 0, 0, 0, 1, 5], None);
        settle(&mut g);
        paid(&mut g, p, "Thor, God of Thunder", [0, 0, 0, 2, 0, 3], None);
        settle(&mut g);
        paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        bind(&mut g, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(stats(&g, thing), (9, 9));
        let mut g = scenario(p);
        let wiccan = paid(
            &mut g,
            p,
            "Wiccan, Rising Magician",
            [0, 1, 0, 0, 0, 4],
            None,
        );
        settle(&mut g);
        let c = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let old = g.object_ref(c);
        paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        bind(&mut g, &[c]);
        settle(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Exile);
        g.move_object(wiccan, Zone::Graveyard);
        step(&mut g, Step::End);
        settle(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
        assert_eq!(g.state.objects[c as usize].controller, p.other());
        assert_ne!(g.object_ref(c), old);

        let mut g = scenario(p);
        let thing = put(&mut g, p, "The Thing, Ben Grimm", Zone::Battlefield);
        let thor = put(&mut g, p, "Thor, God of Thunder", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: thing });
        act(&mut g, Action::Attack { card: thor });
        act(&mut g, Action::FinishAttackers);
        resolve_top(&mut g);
        act(&mut g, Action::FinishBlockers);
        resolve_top(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(thing, CounterKind::PlusOne), 2);
        assert_eq!(g.state.players[p.other().index()].life, 8);
    }
}

#[test]
fn death_finality_poison_ward_and_wolverine_damage_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let c = put(&mut g, p, "Volcanic Villain", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Thunderbolts Conspiracy",
            [0, 0, 1, 0, 0, 3],
            None,
        );
        settle(&mut g);
        paid(
            &mut g,
            p,
            "Lightning Strike",
            [0, 0, 0, 1, 0, 1],
            Some(Target::Card(c)),
        );
        settle(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
        assert!(g.object_view(c).has_subtype("Hero"));
        assert_eq!(g.counter_count(c, CounterKind::Finality), 1);
        paid(
            &mut g,
            p,
            "Lightning Strike",
            [0, 0, 0, 1, 0, 1],
            Some(Target::Card(c)),
        );
        settle(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Exile);
        let mut g = scenario(p);
        let society = paid(&mut g, p, "The Serpent Society", [0, 0, 1, 0, 1, 1], None);
        settle(&mut g);
        g.state.priority = p.other();
        paid(
            &mut g,
            p.other(),
            "Unsummon",
            [0, 1, 0, 0, 0, 0],
            Some(Target::Card(society)),
        );
        resolve_top(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 5);
        assert_eq!(g.object_view(society).zone, Zone::Hand);
        let mut g = scenario(p);
        let enemy = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let w = paid(
            &mut g,
            p,
            "Wolverine, Fierce Fighter",
            [0, 0, 0, 1, 1, 2],
            None,
        );
        resolve_top(&mut g);
        bind(&mut g, &[enemy]);
        settle(&mut g);
        assert_eq!(g.state.objects[w as usize].damage, 4);
        assert_eq!(g.state.objects[enemy as usize].damage, 3);
        paid(
            &mut g,
            p,
            "Truck Toss",
            [0, 0, 0, 2, 0, 2],
            Some(Target::Card(w)),
        );
        settle(&mut g);
        assert_eq!(g.object_view(w).zone, Zone::Battlefield);
        assert_eq!(g.state.objects[w as usize].damage, 4);
    }
}

#[test]
fn modal_crew_white_widow_and_stratagem_groups() {
    for p in [Player::P0, Player::P1] {
        for recover in [false, true] {
            let mut g = scenario(p);
            let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let dead = put(&mut g, p, "Vibranium Energy Daggers", Zone::Graveyard);
            paid(
                &mut g,
                p,
                "White Widow, Free Agent",
                [1, 0, 0, 0, 0, 3],
                None,
            );
            resolve_top(&mut g);
            act(
                &mut g,
                Action::ChooseMode {
                    mode: u16::from(recover),
                },
            );
            let selected = if recover { vec![dead] } else { vec![a, b] };
            bind(&mut g, &selected);
            settle(&mut g);
            if recover {
                assert_eq!(g.object_view(dead).zone, Zone::Hand);
            } else {
                assert_eq!(stats(&g, a), (3, 3));
                assert_eq!(stats(&g, b), (3, 3));
            }
        }
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let discard = put(&mut g, p, "Truck Toss", Zone::Hand);
        let spell = put(&mut g, p, "Trickster's Stratagem", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        if g.state.announcing.is_some() {
            act(
                &mut g,
                Action::PayCosts {
                    payment: [0, 1, 0, 0, 0, 3],
                },
            );
        }
        resolve_top(&mut g);
        let r = g.object_ref(target);
        act(
            &mut g,
            Action::ChooseLibraryDestination {
                object: r,
                destination: LibraryDestination::SecondFromTop,
            },
        );
        act(&mut g, Action::Discard { card: discard });
        settle(&mut g);
        let library = &g.state.players[p.other().index()].library;
        assert_eq!(library[library.len() - 2], target);
        assert_eq!(stats(&g, own), (3, 3));
        let mut g = scenario(p);
        let crew = put(&mut g, p, "The Ruinous Wrecking Crew", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 0];
        act(&mut g, Action::AnnounceCast { card: crew });
        act(&mut g, Action::ChooseX { value: 0 });
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 1, 0, 0],
            },
        );
        resolve_top(&mut g);
        assert_eq!(
            g.legal_actions()
                .into_iter()
                .filter(|a| matches!(a, Action::ChooseMode { .. }))
                .collect::<Vec<_>>(),
            vec![Action::ChooseMode { mode: 0 }]
        );
        act(&mut g, Action::ChooseMode { mode: 0 });
        settle(&mut g);
        assert_eq!(stats(&g, crew), (2, 2));
        assert!(g.state.stack.is_empty());
    }
}

#[test]
fn kingpin_extort_attack_and_hideout_connive() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let king = paid(&mut g, p, "The Kingpin of Crime", [1, 0, 1, 0, 0, 1], None);
        settle(&mut g);
        paid(
            &mut g,
            p,
            "Vibranium Energy Daggers",
            [0, 0, 0, 0, 0, 1],
            None,
        );
        resolve_top(&mut g);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        act(
            &mut g,
            Action::PayResolutionMana {
                payment: [1, 0, 0, 0, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        g.state.objects[king as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: king });
        act(&mut g, Action::FinishAttackers);
        resolve_top(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 19);
        let mut g = scenario(p);
        let villain = put(&mut g, p, "Volcanic Villain", Zone::Battlefield);
        let land = put(&mut g, p, "Villainous Hideout", Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        let discard = put(&mut g, p, "Truck Toss", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        choose(
            &mut g,
            |a| matches!(a,Action::Activate{card,target:Some(Target::Card(c)),payment,..}if *card==land&&*c==villain&&*payment==[0,0,0,0,0,3]),
        );
        resolve_top(&mut g);
        act(&mut g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(stats(&g, villain), (4, 3));
        assert!(g.state.objects[land as usize].tapped);
    }
}

#[test]
fn literal_printed_creature_bodies_and_keyword_families() {
    // Oracle literals, not Card.cost()/rules constants. Event scenarios above
    // separately exercise these permanents' nonkeyword abilities.
    let rows = [
        ("Super-Adaptoid", (1, 2), Keywords::NONE),
        ("Super-Skrull", (4, 5), Keywords::FLYING),
        ("Swordsman, Sharp Scoundrel", (2, 2), Keywords::NONE),
        ("Taskmaster, Mercenary Mimic", (3, 5), Keywords::NONE),
        (
            "Thanos, the Mad Titan",
            (4, 4),
            Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
        ),
        ("The Astonishing Ant-Man", (1, 1), Keywords::NONE),
        ("The Kingpin of Crime", (1, 5), Keywords::NONE),
        ("The Masters of Evil", (5, 6), Keywords::NONE),
        ("The Mighty Thor, Jane Foster", (3, 3), Keywords::FLYING),
        ("The Ruinous Wrecking Crew", (2, 2), Keywords::NONE),
        ("The Scarlet Witch", (2, 3), Keywords::NONE),
        (
            "The Sentry, Golden Guardian",
            (5, 5),
            Keywords::FLYING
                .union(Keywords::VIGILANCE)
                .union(Keywords::INDESTRUCTIBLE),
        ),
        ("The Serpent Society", (3, 4), Keywords::DEATHTOUCH),
        ("The Thing, Ben Grimm", (7, 7), Keywords::TRAMPLE),
        ("The Unbeatable Squirrel Girl", (4, 4), Keywords::NONE),
        (
            "The Vision",
            (2, 5),
            Keywords::FLYING.union(Keywords::VIGILANCE),
        ),
        (
            "The Wondrous Wasp",
            (2, 1),
            Keywords::FLYING.union(Keywords::FLASH),
        ),
        (
            "Thor Odinson",
            (4, 4),
            Keywords::FLYING.union(Keywords::VIGILANCE),
        ),
        ("Thor, God of Thunder", (5, 5), Keywords::FLYING),
        (
            "Tigra, Feline Fury",
            (2, 1),
            Keywords::FLASH.union(Keywords::TRAMPLE),
        ),
        ("Titania, Rugged Rumbler", (5, 5), Keywords::NONE),
        ("Tony Stark", (1, 3), Keywords::NONE),
        ("U.S.Agent, John Walker", (3, 2), Keywords::NONE),
        ("Ultron Drone", (2, 3), Keywords::NONE),
        ("Ultron, Artificial Malevolence", (2, 4), Keywords::NONE),
        ("Undercover Skrull", (1, 1), Keywords::NONE),
        ("Unliving Legionnaire", (3, 2), Keywords::FLYING),
        ("Viv Vision, Teen Synthezoid", (2, 2), Keywords::FLYING),
        ("Volcanic Villain", (3, 2), Keywords::HASTE),
        ("Wakandan Drone Flock", (3, 3), Keywords::FLYING),
        ("Wakandan Royal Guard", (4, 4), Keywords::VIGILANCE),
        ("War Machine, Legacy of Iron", (1, 3), Keywords::FLYING),
        ("Whiplash, Vengeful Engineer", (2, 2), Keywords::NONE),
        ("White Tiger, Ava Ayala", (2, 2), Keywords::NONE),
        ("White Widow, Free Agent", (2, 3), Keywords::NONE),
        ("Wiccan, Rising Magician", (4, 4), Keywords::FLYING),
        (
            "Winter Soldier, Icy Assassin",
            (2, 2),
            Keywords::VIGILANCE.union(Keywords::MENACE),
        ),
        ("Wolverine, Fierce Fighter", (3, 5), Keywords::HASTE),
        ("Wonder Man, Hollywood Hero", (4, 4), Keywords::FLYING),
        ("Yellowjacket, Heartless Marauder", (1, 2), Keywords::FLYING),
    ];
    for p in [Player::P0, Player::P1] {
        for &(name, body, keywords) in &rows {
            let mut g = scenario(p);
            let id = put(&mut g, p, name, Zone::Battlefield);
            assert_eq!(stats(&g, id), body, "{name}");
            assert_eq!(g.keywords(id), keywords, "{name}");
        }
    }
}

#[test]
fn remaining_modal_resolution_and_restricted_mana_clauses() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let token = fixture_token(&mut g, p.other(), "@token/283fd83a9aec83a51b736926");
        let discard = put(&mut g, p, "Swamp", Zone::Hand);
        let crew = put(&mut g, p, "The Ruinous Wrecking Crew", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 4];
        act(&mut g, Action::AnnounceCast { card: crew });
        act(&mut g, Action::ChooseX { value: 4 });
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 1, 0, 4],
            },
        );
        resolve_top(&mut g);
        act(&mut g, Action::ChooseMode { mode: 15 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        bind(&mut g, &[token]);
        resolve_top(&mut g);
        act(&mut g, Action::Discard { card: discard });
        let r = g.object_ref(own);
        act(&mut g, Action::Sacrifice { object: r });
        let r = g.object_ref(enemy);
        act(&mut g, Action::Sacrifice { object: r });
        settle(&mut g);
        assert_eq!(stats(&g, crew), (6, 6));
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert!(!g.on_battlefield(token));
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        let mut g = scenario(p);
        let land = put(&mut g, p, "Villainous Hideout", Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production,..}if *card==land&&*production==[0,0,0,1,0,0]),
        );
        let villain = put(&mut g, p, "Volcanic Villain", Zone::Hand);
        act(
            &mut g,
            Action::Cast {
                card: villain,
                target: None,
                payment: [0, 0, 0, 1, 0, 2],
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, villain), (3, 2));
        g.state.objects[land as usize].tapped = false;
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production,..}if *card==land&&*production==[0,0,0,1,0,0]),
        );
        let toss = put(&mut g, p, "Truck Toss", Zone::Hand);
        assert_eq!(
            g.apply(
                p,
                Action::Cast {
                    card: toss,
                    target: Some(Target::Player(p.other())),
                    payment: [0, 0, 0, 2, 0, 2]
                }
            ),
            Err(Error::IllegalAction)
        );
        let mut g = scenario(p);
        let discarded = put(&mut g, p, "Swamp", Zone::Hand);
        paid(&mut g, p, "Vision of Love", [0, 0, 0, 1, 0, 1], None);
        resolve_top(&mut g);
        act(&mut g, Action::Discard { card: discarded });
        settle(&mut g);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.object_view(discarded).zone, Zone::Graveyard);
        let mut g = scenario(p);
        let card = put(&mut g, p, "Tony Stark", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 4];
        act(&mut g, Action::AnnounceBack { card });
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 1, 0, 4],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(card).name(), "The Invincible Iron Man");
        assert_eq!(stats(&g, card), (5, 5));
        let mut g = scenario(p);
        let equipment = put(&mut g, p, "Vibranium Energy Daggers", Zone::Graveyard);
        let thor = paid(&mut g, p, "Thor, God of Thunder", [0, 0, 0, 2, 0, 3], None);
        resolve_top(&mut g);
        bind(&mut g, &[equipment]);
        settle(&mut g);
        assert_eq!(g.object_view(equipment).zone, Zone::Exile);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceExile { card: equipment });
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        bind(&mut g, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(equipment).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert_eq!(stats(&g, thor), (5, 5));

        let mut g = scenario(p);
        let land = put(&mut g, p, "Villainous Hideout", Zone::Battlefield);
        let villain = put(&mut g, p, "Volcanic Villain", Zone::Battlefield);
        g.state.turn += 1;
        g.state.entered_this_turn = None;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production,..}if *card==land&&*production==[0,0,0,1,0,0]),
        );
        act(
            &mut g,
            Action::Activate {
                card: villain,
                ability: 0,
                target: None,
                payment: [0, 0, 0, 1, 0, 5],
                counter_payment: Counters::default(),
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, villain), (5, 4));
        let mut g = scenario(p);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let r = g.object_ref(target);
        g.change_controller(r, p.other());
        let spell = put(&mut g, p, "Trickster's Stratagem", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 3],
            },
        );
        resolve_top(&mut g);
        assert_eq!(g.actor(), Some(p));
        let r = g.object_ref(target);
        act(
            &mut g,
            Action::ChooseLibraryDestination {
                object: r,
                destination: LibraryDestination::Bottom,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.first(), Some(&target));
    }
}

#[test]
fn printed_event_boundaries_and_optional_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid(&mut g, p, "The Serpent Society", [0, 0, 1, 0, 1, 1], None);
        settle(&mut g);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(own, CounterKind::Deathtouch, 1);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let token = fixture_token(&mut g, p.other(), "@token/b73d837afe1978c0c8507afb");
        paid(
            &mut g,
            p,
            "Lightning Strike",
            [0, 0, 0, 1, 0, 1],
            Some(Target::Card(own)),
        );
        resolve_top(&mut g);
        resolve_top(&mut g);
        assert!(!g.legal_actions().contains(&Action::Sacrifice {
            object: g.object_ref(token)
        }));
        let r = g.object_ref(enemy);
        act(&mut g, Action::Sacrifice { object: r });
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        assert!(g.on_battlefield(token));
        let mut g = scenario(p);
        let king = put(&mut g, p, "The Kingpin of Crime", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: king });
        act(&mut g, Action::FinishAttackers);
        resolve_top(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        resolve_top(&mut g);
        act(&mut g, Action::FinishBlockers);
        resolve_top(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        let mut g = scenario(p);
        let girl = paid(
            &mut g,
            p,
            "The Unbeatable Squirrel Girl",
            [0, 0, 0, 0, 3, 1],
            None,
        );
        settle(&mut g);
        g.state.objects[girl as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: girl });
        act(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(tokens(&g, p, "Squirrel Token").len(), 2);
        let mut g = scenario(p);
        let thanos = put(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
        // Established-permanent fixture: clear the entry-turn witness before full cost.
        g.state.turn += 1;
        g.state.entered_this_turn = None;
        let odd = put(&mut g, p.other(), "Volcanic Villain", Zone::Battlefield);
        let even = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, p, thanos, 0, [1, 1, 1, 1, 1, 1], None);
        g.move_object(thanos, Zone::Exile);
        resolve_top(&mut g);
        act(
            &mut g,
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Odd,
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(odd).zone, Zone::Graveyard);
        assert!(g.on_battlefield(even));
        let mut g = scenario(p);
        let card = put(&mut g, p, "Volcanic Villain", Zone::Hand);
        let saga = paid(&mut g, p, "World War Hulk", [0, 0, 0, 0, 2, 3], None);
        settle(&mut g);
        g.move_object(saga, Zone::Graveyard);
        act(
            &mut g,
            Action::AnnounceFree {
                card,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        act(&mut g, Action::PayCosts { payment: [0; 6] });
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        let second = put(&mut g, p, "Volcanic Villain", Zone::Hand);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceFree{card,..}if *card==second))
        );
        let mut g = scenario(p);
        let donor = put(&mut g, p.other(), "Serra Angel", Zone::Graveyard);
        let task = paid(
            &mut g,
            p,
            "Taskmaster, Mercenary Mimic",
            [0, 1, 1, 0, 0, 2],
            None,
        );
        settle(&mut g);
        g.state.turn += 2;
        g.begin_turn();
        step(&mut g, Step::Main1);
        bind(&mut g, &[donor]);
        settle(&mut g);
        assert_eq!(stats(&g, task), (4, 4));
        assert!(
            g.keywords(task)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );

        for (name, mana, targeted) in [
            ("Super Speed", [0, 0, 0, 1, 0, 0], true),
            ("Super Suit", [0, 1, 0, 0, 0, 1], false),
            ("Super Villain Lockup", [1, 0, 0, 0, 0, 1], false),
            ("Tigra, Feline Fury", [0, 0, 0, 0, 1, 1], false),
        ] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[enemy as usize].tapped = true;
            g.state.active = p.other();
            g.state.priority = p;
            g.state.step = Step::Main2;
            let id = paid(&mut g, p, name, mana, targeted.then_some(Target::Card(own)));
            if name == "Super Suit" || name == "Super Villain Lockup" {
                resolve_top(&mut g);
                bind(&mut g, &[if name == "Super Suit" { own } else { enemy }]);
            }
            settle(&mut g);
            assert!(g.on_battlefield(id));
        }
        let mut g = scenario(p);
        let c = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Super Strength",
            [0, 0, 0, 0, 1, 4],
            Some(Target::Card(c)),
        );
        settle(&mut g);
        g.state.priority = p.other();
        paid(
            &mut g,
            p.other(),
            "Unsummon",
            [0, 1, 0, 0, 0, 0],
            Some(Target::Card(c)),
        );
        resolve_top(&mut g);
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 1];
        act(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Hand);
        let mut g = scenario(p);
        let spare = put(&mut g, p, "Forest", Zone::Hand);
        let titan = put(&mut g, p, "Titania, Rugged Rumbler", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 4];
        act(&mut g, Action::AnnounceCast { card: titan });
        let cost = g
            .observe(p)
            .cost_choices
            .into_iter()
            .find(|c| c.cost.colored == [0, 0, 0, 0, 1, 0] && c.cost.generic == 4)
            .unwrap();
        act(&mut g, Action::ChooseSpellCost { cost: cost.index });
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 1, 4],
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, titan), (5, 5));
        assert_eq!(g.object_view(spare).zone, Zone::Hand);
        let mut g = scenario(p);
        let thanos = put(&mut g, p, "Thanos, the Mad Titan", Zone::Battlefield);
        // Established-permanent fixture: clear the entry-turn witness before full cost.
        g.state.turn += 1;
        g.state.entered_this_turn = None;
        let other = put(&mut g, p.other(), "Volcanic Villain", Zone::Battlefield);
        activate(&mut g, p, thanos, 0, [1, 1, 1, 1, 1, 1], None);
        resolve_top(&mut g);
        act(
            &mut g,
            Action::ChooseManaValueParity {
                parity: ManaValueParity::Odd,
            },
        );
        settle(&mut g);
        assert!(g.on_battlefield(thanos));
        assert_eq!(g.object_view(other).zone, Zone::Graveyard);
    }
}
