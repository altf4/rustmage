//! Focused printed outcomes; shared engine invariants live in core tests.
//! Independent bounded scenarios for MSH identities 141..211.
use super::*;

fn fresh(p: Player) -> Game {
    let mut g = scenario(p);
    for player in &mut g.state.players {
        player.mana = [0; 6];
    }
    g
}
fn mana(g: &mut Game, p: Player, amount: Mana) {
    g.state.players[p.index()].mana = amount;
}
fn until(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if predicate(g) {
            return;
        }
        assert_eq!(
            g.state.pending,
            Pending::Priority,
            "explicit choice required"
        );
        act(g, Action::Pass);
    }
    panic!("24 priority-action budget exhausted");
}
fn activate(g: &mut Game, id: CardId, ability: u16, target: Option<Target>) {
    if let Some(a) = g.legal_actions().into_iter().find(|a| matches!(a,Action::Activate{card,ability:s,target:t,..}if *card==id&&*s==ability&&*t==target)) {
        act(g,a);
    } else {
        act(g,Action::AnnounceActivate { card:id, ability, counter_payment:Counters::default() });
        if let Some(target)=target { act(g,Action::ChooseTarget {target}); }
        if g.legal_actions().contains(&Action::FinishTargets) { act(g,Action::FinishTargets); }
        pay(g);
    }
}
fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
        .map(|(id, _)| id as CardId)
        .collect()
}
fn upkeep(g: &mut Game, p: Player) {
    g.state.active = p;
    g.begin_step(Step::Upkeep);
    g.give_priority();
}
#[test]
fn msh_c_lands_entry_life_and_each_printed_mana_color() {
    for (name, tapped, gain, colors) in [
        ("Mountain", false, 0, vec![3]),
        ("Plains", false, 0, vec![0]),
        ("Pym Technologies", true, 1, vec![4, 1]),
        ("Stark Industries", true, 1, vec![1, 3]),
        ("Subterranean Cavern", true, 1, vec![2, 4]),
    ] {
        {
            let p = Player::P0;
            let mut g = fresh(p);
            let id = put(&mut g, p, name, Zone::Hand);
            act(&mut g, Action::PlayLand { card: id });
            settle(&mut g);
            assert_eq!(g.object_view(id).tapped, tapped, "{name}");
            assert_eq!(g.state.players[p.index()].life, 20 + gain, "{name}");
            for &color in &colors {
                let mut g = fresh(p);
                let id = put(&mut g, p, name, Zone::Battlefield);
                let mut expected = [0; 6];
                expected[color] = 1;
                if g.legal_actions().contains(&Action::TapMana { card: id }) {
                    act(&mut g, Action::TapMana { card: id });
                } else {
                    choose(
                        &mut g,
                        |a| matches!(a,Action::ActivateMana{card,production,..}if *card==id&&*production==expected),
                    );
                }
                if g.state.players[p.index()].mana == [0; 6] {
                    act(
                        &mut g,
                        Action::ChooseMana {
                            production: expected,
                        },
                    );
                }
                assert_eq!(g.state.players[p.index()].mana, expected);
                assert!(g.object_view(id).tapped);
            }
        }
    }
}

#[test]
fn msh_c_entry_tokens_and_one_or_more_batch_draw() {
    {
        let p = Player::P0;
        for (name, cost, n) in [
            ("Okoye, Dora Milaje Leader", [1, 0, 0, 0, 0, 3], 2),
            ("S.H.I.E.L.D. Deployment Drone", [0, 1, 0, 0, 0, 2], 1),
            ("S.H.I.E.L.D. Helicarrier", [0, 0, 0, 0, 0, 4], 2),
        ] {
            let mut g = fresh(p);
            let id = put(&mut g, p, name, Zone::Hand);
            mana(&mut g, p, cost);
            cast(&mut g, id, None);
            settle(&mut g);
            let ts = tokens(&g, p);
            assert_eq!(ts.len(), n, "{name}");
            for t in ts {
                assert_eq!(stats(&g, t), (1, 1));
                assert!(g.object_view(t).summoning_sick);
            }
        }
        let mut g = fresh(p);
        let okoye = put(&mut g, p, "Okoye, Dora Milaje Leader", Zone::Hand);
        mana(&mut g, p, [1, 0, 0, 0, 0, 3]);
        cast(&mut g, okoye, None);
        settle(&mut g);
        let soldiers = tokens(&g, p);
        assert!(!g.keywords(soldiers[0]).contains(Keywords::FIRST_STRIKE));
        g.state.turn += 1;
        g.begin_turn();
        start_attack(&mut g, soldiers[0]);
        assert!(g.keywords(soldiers[0]).contains(Keywords::FIRST_STRIKE));
        assert!(!g.keywords(soldiers[1]).contains(Keywords::FIRST_STRIKE));
        assert!(!g.keywords(okoye).contains(Keywords::FIRST_STRIKE));
        finish_unblocked(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        let mut g = fresh(p);
        let reed = put(
            &mut g,
            p,
            "Mister Fantastic, Reed Richards",
            Zone::Battlefield,
        );
        assert_eq!(stats(&g, reed), (2, 4));
        assert!(g.keywords(reed).contains(Keywords::REACH));
        let ship = put(&mut g, p, "S.H.I.E.L.D. Helicarrier", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 0, 0, 4]);
        cast(&mut g, ship, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        act(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(tokens(&g, p).len(), 2);
    }
}

#[test]
fn msh_c_artifact_entry_and_villain_cast_observers() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let a = put(&mut g, p, "Machinesmith Automaton", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 1, 0, 2]);
        cast(&mut g, a, None);
        settle(&mut g);
        assert_eq!(stats(&g, a), (2, 2));
        let drone = put(&mut g, p, "S.H.I.E.L.D. Deployment Drone", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 2]);
        cast(&mut g, drone, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::OrderTriggers { .. })
        });
        let trigger = g
            .state
            .ordering_triggers
            .iter()
            .find(|t| t.source.is_some_and(|identity| identity.id == a))
            .unwrap()
            .id;
        act(&mut g, Action::PutTrigger { id: trigger });
        if matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            let trigger = g
                .state
                .ordering_triggers
                .iter()
                .find(|t| t.source.is_some_and(|identity| identity.id == drone))
                .unwrap()
                .id;
            act(&mut g, Action::PutTrigger { id: trigger });
        }
        settle(&mut g);
        assert_eq!(stats(&g, a), (3, 3));
        assert!(g.keywords(a).contains(Keywords::TRAMPLE));
        let mut g = fresh(p);
        put(&mut g, p, "Madame Hydra", Zone::Battlefield);
        let id = put(&mut g, p, "Machinesmith Automaton", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 1, 0, 2]);
        cast(&mut g, id, None);
        settle(&mut g);
        let ts = tokens(&g, p);
        assert_eq!(ts.len(), 1);
        assert_eq!(stats(&g, ts[0]), (2, 1));
        assert!(g.keywords(ts[0]).contains(Keywords::MENACE));
    }
}

#[test]
fn msh_c_connive_land_and_nonland_second_draw_boundaries() {
    {
        let p = Player::P0;
        for (name, cost, base) in [
            ("Red Room Recruit", [0, 0, 1, 0, 0, 1], (1, 2)),
            ("Madame Masque", [0, 0, 1, 0, 0, 4], (3, 2)),
        ] {
            for (discard, bonus) in [("Forest", 0), ("Shock", 1)] {
                let mut g = fresh(p);
                let fodder = put(&mut g, p, discard, Zone::Hand);
                let id = put(&mut g, p, name, Zone::Hand);
                mana(&mut g, p, cost);
                cast(&mut g, id, None);
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectDiscard { .. })
                });
                act(&mut g, Action::Discard { card: fodder });
                settle(&mut g);
                assert_eq!(stats(&g, id), (base.0 + bonus, base.1 + bonus));
                assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
                if name == "Madame Masque" {
                    assert!(tokens(&g, p).is_empty());
                    for _ in 0..2 {
                        let pym = put(&mut g, p, "Pym Particles", Zone::Hand);
                        mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
                        cast(&mut g, pym, Some(Target::Card(id)));
                        settle(&mut g);
                        assert_eq!(tokens(&g, p).len(), 1);
                    }
                }
            }
        }
    }
}

#[test]
fn msh_c_power_up_literal_entry_discounts_lifetime_use_and_outcomes() {
    {
        let p = Player::P0;
        for (name, cast_cost, activation, expected, counter) in [
            (
                "Ninja of the Hand",
                [0, 0, 1, 0, 0, 2],
                [0, 0, 0, 0, 0, 2],
                (3, 3),
                1,
            ),
            (
                "Pet Avengers",
                [0, 0, 0, 0, 1, 3],
                [0, 0, 0, 0, 0, 3],
                (5, 5),
                1,
            ),
            (
                "Serpent Specialist",
                [0, 0, 0, 0, 1, 0],
                [0, 0, 0, 0, 0, 3],
                (3, 3),
                2,
            ),
            (
                "She-Hulk, Jade Defender",
                [0, 0, 0, 0, 1, 3],
                [0, 0, 0, 0, 1, 1],
                (5, 5),
                1,
            ),
            (
                "Quicksilver, Brash Blur",
                [0, 0, 0, 1, 0, 0],
                [0, 0, 0, 0, 0, 4],
                (2, 2),
                1,
            ),
        ] {
            let mut g = fresh(p);
            let id = put(&mut g, p, name, Zone::Hand);
            mana(&mut g, p, cast_cost);
            cast(&mut g, id, None);
            settle(&mut g);
            let discarded = put(&mut g, p.other(), "Forest", Zone::Hand);
            mana(&mut g, p, activation);
            activate(&mut g, id, 0, None);
            if name == "Ninja of the Hand" {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectDiscard { .. })
                });
                act(&mut g, Action::Discard { card: discarded });
            }
            settle(&mut g);
            assert_eq!(stats(&g, id), expected, "{name}");
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), counter);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,ability:0,..}|Action::AnnounceActivate{card,ability:0,..}if *card==id)),"{name} once per object");
            if name == "Pet Avengers" {
                let t = tokens(&g, p);
                assert_eq!(t.len(), 1);
                assert_eq!(stats(&g, t[0]), (3, 2));
                assert!(g.keywords(t[0]).contains(Keywords::VIGILANCE));
            }
            if name == "Quicksilver, Brash Blur" {
                assert!(
                    g.keywords(id)
                        .contains(Keywords::HASTE.union(Keywords::DOUBLE_STRIKE))
                );
                g.finish_cleanup();
                assert!(g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
            }
        }
    }
}

#[test]
fn msh_c_full_price_power_up_and_opening_hand_optional_boundary() {
    {
        let p = Player::P0;
        for (name, full, expected) in [
            ("Ninja of the Hand", [0, 0, 1, 0, 0, 4], (3, 3)),
            ("Pet Avengers", [0, 0, 0, 0, 1, 6], (5, 5)),
            ("Serpent Specialist", [0, 0, 0, 0, 1, 3], (3, 3)),
            ("She-Hulk, Jade Defender", [0, 0, 0, 0, 2, 4], (5, 5)),
            ("Quicksilver, Brash Blur", [0, 0, 0, 1, 0, 4], (2, 2)),
        ] {
            let mut g = fresh(p);
            let id = put(&mut g, p, name, Zone::Battlefield);
            g.state.turn += 1; // Historical entry: no entry-turn discount.
            g.begin_turn();
            let discard = put(&mut g, p.other(), "Forest", Zone::Hand);
            mana(&mut g, p, full);
            activate(&mut g, id, 0, None);
            if name == "Ninja of the Hand" {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectDiscard { .. })
                });
                act(&mut g, Action::Discard { card: discard });
            }
            settle(&mut g);
            assert_eq!(stats(&g, id), expected, "{name} historical full price");
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let object = g.object_ref(id);
            g.change_controller(object, p.other());
            g.state.turn += 1;
            g.state.active = p.other();
            g.begin_turn();
            settle(&mut g);
            mana(&mut g, p.other(), full);
            assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,ability:0,..}|Action::AnnounceActivate{card,ability:0,..}if *card==id)),"{name} once survives turn/control change");
        }
        for target_name in ["S.H.I.E.L.D. Spy Kit", "Political Triumph"] {
            let mut g = fresh(p);
            let target = put(&mut g, p.other(), target_name, Zone::Battlefield);
            let hulk = put(&mut g, p, "She-Hulk, Jade Defender", Zone::Battlefield);
            mana(&mut g, p, [0, 0, 0, 0, 2, 4]);
            activate(&mut g, hulk, 0, Some(Target::Card(target)));
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            assert_eq!(stats(&g, hulk), (5, 5));
        }
        for accept in [false, true] {
            let mut decks = [vec![Card::Forest; 7], vec![Card::Forest; 7]];
            decks[p.index()] = vec![Card::from_name("Quicksilver, Brash Blur").unwrap(); 7];
            let mut g = Game::new(Config {
                decks,
                seed: 20261023,
                starting_player: p,
                deck_rules: DeckRules::Freeform,
            })
            .unwrap();
            act(&mut g, Action::Keep);
            act(&mut g, Action::Keep);
            assert!(matches!(g.state.pending, Pending::OpeningHand { .. }));
            let quick = g
                .state
                .objects
                .iter()
                .position(|o| o.owner == p && o.zone == Zone::Hand)
                .unwrap() as CardId;
            if accept {
                let object = g.object_ref(quick);
                act(&mut g, Action::BeginOnBattlefield { object });
                assert!(g.on_battlefield(quick));
                assert!(g.keywords(quick).contains(Keywords::HASTE));
            }
            act(&mut g, Action::FinishOpeningHand);
            act(&mut g, Action::FinishOpeningHand);
            assert_eq!(g.on_battlefield(quick), accept);
        }
    }
}

#[test]
fn msh_c_stature_variable_zero_and_two_counters() {
    {
        let p = Player::P0;
        for x in [0, 2] {
            let mut g = fresh(p);
            let id = put(&mut g, p, "Stature, Size Shifter", Zone::Hand);
            mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
            cast(&mut g, id, None);
            settle(&mut g);
            mana(&mut g, p, [0, 1, 0, 0, 0, x]);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            settle(&mut g);
            assert_eq!(stats(&g, id), (1 + i128::from(x), 1 + i128::from(x)));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn msh_c_paid_tap_discard_draw_and_graveyard_return() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let misty = put(&mut g, p, "Misty Knight, Hero for Hire", Zone::Battlefield);
        let fodder = put(&mut g, p, "Forest", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 0, 0, 2]);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: misty,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(fodder);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(misty).tapped);
        assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), 1);
        let pounce = put(&mut g, p, "Panther Pounce", Zone::Hand);
        mana(&mut g, p, [1, 0, 0, 0, 0, 0]);
        announce(
            &mut g,
            pounce,
            &[Target::Player(p.other()), Target::Card(misty)],
        );
        settle(&mut g);
        let second = put(&mut g, p, "Forest", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 0, 0, 2]);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: misty,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(second);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert_eq!(g.hand_count(p), 3);
        let mut g = fresh(p);
        let id = put(&mut g, p, "Project Deathlok Soldier", Zone::Graveyard);
        mana(&mut g, p, [0, 0, 1, 0, 0, 2]);
        activate(&mut g, id, 0, None);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn msh_c_moonstone_discard_permission_source_independence_and_expiry() {
    {
        let p = Player::P0;
        for (name, accept, play) in [
            ("Shock", true, true),
            ("Forest", true, true),
            ("Shock", false, false),
            ("Shock", true, false),
        ] {
            let mut g = fresh(p);
            let moon = put(&mut g, p, "Moonstone, Harsh Mistress", Zone::Battlefield);
            let misty = put(&mut g, p, "Misty Knight, Hero for Hire", Zone::Battlefield);
            let card = put(&mut g, p, name, Zone::Hand);
            mana(&mut g, p, [0, 0, 0, 0, 0, 2]);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: misty,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let object = g.object_ref(card);
            act(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            act(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.object_view(card).zone,
                if accept { Zone::Exile } else { Zone::Graveyard }
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            // Production movement is a source-departure boundary fixture, not a spell claim.
            g.move_object(moon, Zone::Graveyard);
            if play {
                if name == "Forest" {
                    act(&mut g, Action::PlayLand { card });
                    assert!(g.on_battlefield(card));
                } else {
                    assert!(!g.legal_actions().contains(&Action::AnnounceExile { card }));
                    mana(&mut g, p, [0, 0, 0, 1, 0, 0]);
                    act(&mut g, Action::AnnounceExile { card });
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Player(p.other()),
                        },
                    );
                    pay(&mut g);
                    settle(&mut g);
                    assert_eq!(g.state.players[p.other().index()].life, 18);
                }
            } else if accept {
                g.finish_cleanup(); // Opponent's turn; permission survives.
                assert!(
                    g.state
                        .exile_permissions
                        .iter()
                        .any(|row| row.object.card == card)
                );
                g.finish_cleanup(); // Beginning of the next actual own turn.
                mana(&mut g, p, [0, 0, 0, 1, 0, 0]);
                assert!(g.legal_actions().contains(&Action::AnnounceExile { card }));
                g.finish_cleanup(); // That own turn's cleanup expires the permission.
                g.state.active = p;
                g.state.priority = p;
                g.state.step = Step::Main1;
                mana(&mut g, p, [0, 0, 0, 1, 0, 0]);
                assert!(!g.legal_actions().contains(&Action::AnnounceExile { card }));
                assert_eq!(g.object_view(card).zone, Zone::Exile);
            }
        }
    }
}

#[test]
fn msh_c_milling_selection_filters_and_exact_amounts() {
    {
        let p = Player::P0;
        for (rick, select) in [(false, true), (false, false), (true, true)] {
            let mut g = fresh(p);
            g.state.players[p.index()].library.clear();
            let ineligible = put(&mut g, p, "Shock", Zone::Library);
            let eligible = put(
                &mut g,
                p,
                if rick { "Super Intelligence" } else { "Forest" },
                Zone::Library,
            );
            if rick {
                put(&mut g, p, "Forest", Zone::Library);
                put(&mut g, p, "Island", Zone::Library);
            }
            if rick {
                let id = put(
                    &mut g,
                    p,
                    "Rick Jones, Destined Sidekick",
                    Zone::Battlefield,
                );
                mana(&mut g, p, [0, 0, 0, 0, 0, 3]);
                activate(&mut g, id, 0, None);
            } else {
                let id = put(&mut g, p, "Rapid Rescue", Zone::Hand);
                mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
                cast(&mut g, id, None);
            }
            until(&mut g, |g| {
                matches!(
                    g.state.pending,
                    Pending::RecoverMilled { .. } | Pending::MilledSelection { .. }
                )
            });
            assert_eq!(g.state.players[p.index()].library.len(), 0);
            assert_eq!(g.object_view(ineligible).zone, Zone::Graveyard);
            assert!(!g.legal_actions().contains(&Action::RecoverMilled {
                object: Some(g.object_ref(ineligible))
            }));
            let selection = select.then(|| g.object_ref(eligible));
            if rick {
                if let Some(object) = selection {
                    act(&mut g, Action::SelectMilled { object });
                } else {
                    act(&mut g, Action::FinishMilledSelection);
                }
            } else {
                act(&mut g, Action::RecoverMilled { object: selection });
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(eligible).zone,
                if select { Zone::Hand } else { Zone::Graveyard }
            );
            assert_eq!(g.state.players[p.index()].life, if rick { 20 } else { 22 });
        }
    }
}

#[test]
fn msh_c_second_draw_base_stats_once_per_turn_and_targeted_cast_triggers() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let moon = put(&mut g, p, "Moon Girl and Devil Dinosaur", Zone::Battlefield);
        for expected_hand in [1, 1] {
            let kit = put(&mut g, p, "S.H.I.E.L.D. Spy Kit", Zone::Hand);
            mana(&mut g, p, [1, 0, 0, 0, 0, 0]);
            cast(&mut g, kit, None);
            settle(&mut g);
            assert_eq!(g.hand_count(p), expected_hand);
        }
        let pym = put(&mut g, p, "Pym Particles", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
        cast(&mut g, pym, Some(Target::Card(moon)));
        settle(&mut g);
        assert_eq!(stats(&g, moon), (6, 6));
        assert!(g.keywords(moon).contains(Keywords::TRAMPLE));
        g.finish_cleanup();
        assert_eq!(stats(&g, moon), (2, 2));
        assert!(!g.keywords(moon).contains(Keywords::TRAMPLE));
        for name in [
            "Mockingbird, Ace Agent",
            "Ms. Marvel, Kamala Khan",
            "Storm, Windrider",
        ] {
            let mut g = fresh(p);
            let hero = put(&mut g, p, name, Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let pym = put(&mut g, p, "Pym Particles", Zone::Hand);
            mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
            cast(&mut g, pym, Some(Target::Card(bear)));
            settle(&mut g);
            match name {
                "Mockingbird, Ace Agent" => assert_eq!(stats(&g, hero), (3, 3)),
                "Ms. Marvel, Kamala Khan" => {
                    assert_eq!(g.hand_count(p), 2);
                    assert_eq!(stats(&g, hero), (2, 4));
                    assert!(g.no_maximum_hand_size(p));
                }
                _ => assert!(g.keywords(bear).contains(Keywords::FLYING)),
            }
            g.finish_cleanup();
            assert!(!g.keywords(bear).contains(Keywords::FLYING));
            if name == "Ms. Marvel, Kamala Khan" {
                assert_eq!(stats(&g, hero), (1, 4));
            }
        }
    }
}

#[test]
fn msh_c_namor_symbol_count_and_incursion_nontoken_nonlegendary_copies() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let namor = put(&mut g, p, "Namor the Sub-Mariner", Zone::Hand);
        mana(&mut g, p, [0, 2, 0, 0, 0, 1]);
        cast(&mut g, namor, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
        assert_eq!(stats(&g, namor), (1, 4));
        let spell = put(&mut g, p, "Multiversal Incursion", Zone::Hand);
        mana(&mut g, p, [0, 2, 0, 0, 0, 5]);
        cast(&mut g, spell, None);
        settle(&mut g);
        let ts = tokens(&g, p);
        assert_eq!(ts.len(), 4);
        assert_eq!(stats(&g, namor), (4, 4));
        assert_eq!(ts.iter().filter(|&&id| stats(&g, id) == (1, 1)).count(), 2);
        assert_eq!(ts.iter().filter(|&&id| stats(&g, id) == (2, 2)).count(), 1);
        assert_eq!(ts.iter().filter(|&&id| stats(&g, id) == (4, 4)).count(), 1);
        assert!(ts.iter().all(|&id| !g.object_view(id).is_legendary()));
        assert!(tokens(&g, p.other()).is_empty());
    }
}

#[test]
fn msh_c_quake_targets_and_reptil_forms_and_raft_power_boundary() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let quake = put(&mut g, p, "Quake, Agent of S.H.I.E.L.D.", Zone::Battlefield);
        let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let pym = put(&mut g, p, "Pym Particles", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
        cast(&mut g, pym, Some(Target::Card(quake)));
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        settle(&mut g);
        assert!(g.object_view(land).tapped);
        for (slot, cost, expected, words) in [
            (0, 3, (3, 5), Keywords::REACH.union(Keywords::VIGILANCE)),
            (1, 6, (6, 6), Keywords::TRAMPLE),
        ] {
            let mut g = fresh(p);
            let reptil = put(&mut g, p, "Reptil, Dinomorpher", Zone::Battlefield);
            mana(&mut g, p, [0, 0, 0, 0, 0, cost]);
            activate(&mut g, reptil, slot, None);
            settle(&mut g);
            assert_eq!(stats(&g, reptil), expected);
            assert!(g.keywords(reptil).contains(words));
            assert!(g.has_subtype(reptil, "Dinosaur"));
            assert!(!g.has_subtype(reptil, "Human"));
            g.finish_cleanup();
            assert_eq!(stats(&g, reptil), (1, 2));
            assert!(g.has_subtype(reptil, "Human"));
        }
        for (target_name, cost) in [("Misty Knight, Hero for Hire", 1), ("Pet Avengers", 2)] {
            let mut g = fresh(p);
            let raft = put(&mut g, p, "Raft Security Officer", Zone::Battlefield);
            let target = put(&mut g, p.other(), target_name, Zone::Battlefield);
            mana(&mut g, p, [0, 0, 0, 0, 0, cost]);
            activate(&mut g, raft, 0, Some(Target::Card(target)));
            settle(&mut g);
            assert!(g.object_view(target).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn msh_c_mole_graveyard_land_play_and_moloid_optional_mill() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        put(&mut g, p, "Mole Man, Moloid Master", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        act(&mut g, Action::PlayLand { card: land });
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        let ts = tokens(&g, p);
        assert_eq!(ts.len(), 1);
        let t = ts[0];
        assert_eq!(stats(&g, t), (1, 1));
        assert!(g.has_subtype(t, "Minion"));
        assert_eq!(g.object_view(t).name, "Moloid");
        let another = put(&mut g, p, "Mountain", Zone::Graveyard);
        assert!(
            !g.legal_actions()
                .contains(&Action::PlayLand { card: another })
        );
        g.state.turn += 1;
        g.begin_turn();
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        let before = g.state.players[p.index()].library.len();
        act(&mut g, Action::Attack { card: t });
        act(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        act(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}

#[test]
fn msh_c_vehicle_crew_thresholds_and_car_delayed_owner_return() {
    {
        let p = Player::P0;
        for (vehicle, minimum, expected) in [
            ("S.H.I.E.L.D. Flying Car", 1, (3, 3)),
            ("S.H.I.E.L.D. Helicarrier", 6, (4, 5)),
        ] {
            let mut g = fresh(p);
            let id = put(&mut g, p, vehicle, Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let pet = put(&mut g, p, "Pet Avengers", Zone::Battlefield);
            assert!(!g.is_creature(id));
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let object = g.object_ref(bear);
            act(&mut g, Action::ChooseCrewer { object });
            if minimum == 6 {
                assert!(!g.legal_actions().contains(&Action::FinishCrew));
                let object = g.object_ref(pet);
                act(&mut g, Action::ChooseCrewer { object });
            }
            act(&mut g, Action::FinishCrew);
            settle(&mut g);
            assert!(g.is_creature(id));
            assert_eq!(stats(&g, id), expected);
            assert!(g.keywords(id).contains(Keywords::FLYING));
            assert!(g.object_view(bear).tapped);
            g.finish_cleanup();
            assert!(!g.is_creature(id));
        }
        let mut g = fresh(p);
        let borrowed = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let r = g.object_ref(borrowed);
        g.change_controller(r, p);
        let car = put(&mut g, p, "S.H.I.E.L.D. Flying Car", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 2]);
        g.state.active = p.other();
        g.state.step = Step::Main2;
        cast(&mut g, car, None);
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(borrowed),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(borrowed).zone, Zone::Exile);
        g.begin_step(Step::End);
        g.give_priority();
        until(&mut g, |g| g.on_battlefield(borrowed));
        assert_eq!(g.object_view(borrowed).controller, p.other());
        assert_ne!(g.object_ref(borrowed), r);
    }
}

#[test]
fn msh_c_damage_source_history_discount_and_flash_destroy() {
    {
        let p = Player::P0;
        for reduced in [false, true] {
            let mut g = fresh(p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let dinosaur = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
            if reduced {
                put(&mut g, p, "Ninja of the Hand", Zone::Graveyard);
                put(&mut g, p, "Serpent Specialist", Zone::Graveyard);
            }
            let punch = put(&mut g, p, "Punishing Punch", Zone::Hand);
            mana(&mut g, p, [0, 0, 0, 0, 1, if reduced { 0 } else { 2 }]);
            announce(&mut g, punch, &[Target::Card(bear), Target::Card(dinosaur)]);
            settle(&mut g);
            assert_eq!(g.object_view(dinosaur).damage, 4);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let guardian = put(&mut g, p.other(), "Red Guardian, Super-Soldier", Zone::Hand);
            mana(&mut g, p.other(), [1, 0, 0, 0, 0, 2]);
            act(&mut g, Action::Pass);
            cast(&mut g, guardian, None);
            until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
            assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(dinosaur)
            }));
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
            assert_eq!(g.object_view(dinosaur).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn msh_c_nurse_recent_permanent_return_excludes_old_graveyard_and_spells() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let old = put(&mut g, p, "Forest", Zone::Graveyard);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 1, 0, 0]);
        cast(&mut g, shock, Some(Target::Card(bear)));
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        let nurse = put(&mut g, p, "Night Nurse, Healer of Heroes", Zone::Hand);
        mana(&mut g, p, [1, 0, 0, 0, 0, 1]);
        cast(&mut g, nurse, None);
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        for invalid in [old, shock] {
            assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(invalid)
            }));
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert!(
            g.keywords(nurse)
                .contains(Keywords::FLASH.union(Keywords::LIFELINK))
        );
    }
}

#[test]
fn msh_c_upkeep_modal_counter_payment_and_each_counter_kind() {
    {
        let p = Player::P0;
        for mode in [0, 1] {
            let mut g = fresh(p);
            let hyde = put(&mut g, p, "Mister Hyde, Monster Within", Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[bear as usize].counters.stun = 1;
            upkeep(&mut g, p);
            until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
            act(&mut g, Action::ChooseMode { mode });
            if mode == 1 {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectChoice { .. })
                });
                let object = g.object_ref(bear);
                act(
                    &mut g,
                    Action::ChooseCreatureCounterCost {
                        object,
                        counter: CounterKind::Stun,
                    },
                );
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(hyde, CounterKind::PlusOne),
                u64::from(mode == 0)
            );
            assert_eq!(g.hand_count(p), usize::from(mode == 1));
            assert_eq!(
                g.counter_count(bear, CounterKind::Stun),
                u64::from(mode == 0)
            );
        }
        let mut g = fresh(p);
        put(&mut g, p, "Mister Hyde, Monster Within", Zone::Battlefield);
        upkeep(&mut g, p);
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        act(&mut g, Action::ChooseMode { mode: 1 });
        settle(&mut g);
        assert_eq!(g.hand_count(p), 0);
        let mut g = fresh(p);
        let broker = put(&mut g, p, "Powerful Broker", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].counters.plus_one = 2;
        g.state.objects[bear as usize].counters.stun = 3;
        activate(&mut g, broker, 0, Some(Target::Card(bear)));
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 3);
        assert_eq!(g.counter_count(bear, CounterKind::Stun), 4);
        let mut g = fresh(p);
        let broker = put(&mut g, p, "Powerful Broker", Zone::Battlefield);
        g.state.players[p.other().index()].poison_counters = 2;
        activate(&mut g, broker, 0, Some(Target::Player(p.other())));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].poison_counters, 3);
    }
}

fn start_attack(g: &mut Game, id: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
}
fn finish_unblocked(g: &mut Game) {
    until(g, |g| g.state.pending == Pending::Blockers);
    act(g, Action::FinishBlockers);
    until(g, |g| {
        g.state.step == Step::Main2 && g.state.pending == Pending::Priority
    });
}
#[test]
fn msh_c_equipment_actual_entry_attachment_worthy_and_attack_effects() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let kit = put(&mut g, p, "S.H.I.E.L.D. Spy Kit", Zone::Hand);
        mana(&mut g, p, [1, 0, 0, 0, 0, 0]);
        cast(&mut g, kit, None);
        settle(&mut g);
        mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
        activate(&mut g, kit, 0, Some(Target::Card(bear)));
        settle(&mut g);
        assert_eq!(stats(&g, bear), (3, 3));
        start_attack(&mut g, bear);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrary { .. })
        });
        assert!(!g.object_view(bear).tapped);
        let top = *g.state.players[p.index()].library.last().unwrap();
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Bottom,
            },
        );
        finish_unblocked(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        assert_eq!(g.state.players[p.index()].library.first(), Some(&top));
        let mut g = fresh(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let equipment = put(&mut g, p, "Stolen Stark Tech", Zone::Hand);
        mana(&mut g, p, [0, 0, 1, 0, 0, 1]);
        g.state.active = p.other();
        g.state.step = Step::Main2;
        cast(&mut g, equipment, None);
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, bear), (3, 2));
        assert!(g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 1, 0, 0]);
        if g.actor() != Some(p) {
            act(&mut g, Action::Pass);
        }
        cast(&mut g, shock, Some(Target::Card(bear)));
        settle(&mut g);
        assert!(g.on_battlefield(bear));
        g.finish_cleanup();
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        let mut g = fresh(p);
        let hero = put(&mut g, p, "Quicksilver, Brash Blur", Zone::Battlefield);
        let villain = put(&mut g, p, "Red Hulk", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
        let hammer = put(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 1, 0, 3]);
        cast(&mut g, hammer, None);
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(enemy).damage, 4);
        mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==hammer&&*t==villain)));
        activate(&mut g, hammer, 0, Some(Target::Card(hero)));
        settle(&mut g);
        start_attack(&mut g, hero);
        finish_unblocked(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        let mut g = fresh(p);
        let hammer = put(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Hand);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        mana(&mut g, p, [0, 0, 0, 1, 0, 3]);
        cast(&mut g, hammer, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        act(&mut g, Action::FinishTargets);
        settle(&mut g);
        assert!(g.on_battlefield(hammer));
        let mut g = fresh(p);
        let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let equipment = put(&mut g, p, "Stolen Stark Tech", Zone::Battlefield);
        mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
        activate(&mut g, equipment, 0, Some(Target::Card(bearer)));
        settle(&mut g);
        assert_eq!(stats(&g, bearer), (3, 2));
        assert!(!g.keywords(bearer).contains(Keywords::INDESTRUCTIBLE));
        let mut g = fresh(p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let hammer = put(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 1, 0, 2]);
        activate(&mut g, hammer, 1, None);
        settle(&mut g);
        for id in [a, b, hammer] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn msh_c_treasure_and_restricted_creature_equipment_mana() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let executive = put(&mut g, p, "Stark Industries Executive", Zone::Battlefield);
        mana(&mut g, p, [0, 0, 0, 0, 0, 2]);
        activate(&mut g, executive, 0, None);
        settle(&mut g);
        let ts = tokens(&g, p);
        assert_eq!(ts.len(), 1);
        let treasure = ts[0];
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production,..}if *card==treasure&&*production==[0,0,0,0,1,0]),
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert!(!g.on_battlefield(treasure));
        let mut g = fresh(p);
        let ronin = put(&mut g, p, "Ronin, Shadow Stalker", Zone::Battlefield);
        let nurse = put(&mut g, p, "Night Nurse, Healer of Heroes", Zone::Hand);
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production,..}if *card==ronin&&*production==[2,0,0,0,0,0]),
        );
        assert_eq!(g.state.players[p.index()].life, 18);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: nurse })
        );
        let kit = put(&mut g, p, "S.H.I.E.L.D. Spy Kit", Zone::Hand);
        cast(&mut g, kit, None);
        settle(&mut g);
        activate(&mut g, kit, 0, Some(Target::Card(ronin)));
        settle(&mut g);
        let enemy = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ronin,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        let object = g.object_ref(kit);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(kit).zone, Zone::Graveyard);
        assert_eq!(stats(&g, enemy), (3, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, enemy), (7, 6));
        let mut g = fresh(p);
        let shang = put(&mut g, p, "Shang-Chi, Master of Kung Fu", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 0, 1, 1]);
        cast(&mut g, shang, None);
        settle(&mut g);
        let patriot = put(&mut g, p, "Patriot, Shield Wielder", Zone::Hand);
        mana(&mut g, p, [1, 0, 0, 0, 0, 1]);
        cast(&mut g, patriot, None);
        settle(&mut g);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production,..}if *card==shang&&*production==[0,2,0,0,0,0]),
        );
        let spell = put(&mut g, p, "Pym Particles", Zone::Hand);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        activate(&mut g, patriot, 0, Some(Target::Card(bear)));
        settle(&mut g);
        assert!(g.object_view(patriot).tapped);
        assert_eq!(stats(&g, bear), (4, 2));
        assert!(!g.keywords(patriot).contains(Keywords::HASTE));
    }
}

#[test]
fn msh_c_successful_counter_and_tap_reflexive_actions_announce_targets_later() {
    {
        let p = Player::P0;
        for remove_before_counter in [false, true] {
            let mut g = fresh(p);
            let hulk = put(&mut g, p, "Red Hulk", Zone::Battlefield);
            put(&mut g, p, "Doubling Season", Zone::Battlefield);
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            mana(&mut g, p, [0, 0, 0, 1, 0, 0]);
            cast(&mut g, shock, Some(Target::Card(hulk)));
            if remove_before_counter {
                until(&mut g, |g| g.object_view(shock).zone == Zone::Graveyard);
            } else {
                until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
                assert_eq!(g.counter_count(hulk, CounterKind::PlusOne), 2);
                assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(hulk)
                }));
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
            }
            let murder = put(&mut g, p, "Murder", Zone::Hand);
            mana(&mut g, p, [0, 0, 2, 0, 0, 1]);
            cast(&mut g, murder, Some(Target::Card(hulk)));
            settle(&mut g);
            assert_eq!(g.object_view(hulk).zone, Zone::Graveyard);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if remove_before_counter { 20 } else { 18 }
            );
        }
        for accept in [false, true] {
            let mut g = fresh(p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spider = put(&mut g, p, "Spider-Man, To the Rescue", Zone::Hand);
            mana(&mut g, p, [0, 0, 0, 0, 1, 2]);
            cast(&mut g, spider, None);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectOptional { .. })
            });
            assert!(!g.object_view(spider).tapped);
            act(&mut g, Action::ChooseOptional { accept });
            if accept {
                until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
                assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(spider)
                }));
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bear),
                    },
                );
            }
            settle(&mut g);
            assert_eq!(g.object_view(spider).tapped, accept);
            assert_eq!(g.keywords(bear).contains(Keywords::INDESTRUCTIBLE), accept);
            g.finish_cleanup();
            assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}

#[test]
fn msh_c_haste_power_and_flying_blocker_restrictions_use_current_creatures() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let speed = put(&mut g, p, "Speed, Young Avenger", Zone::Battlefield);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let fast = put(
            &mut g,
            p.other(),
            "Quicksilver, Brash Blur",
            Zone::Battlefield,
        );
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 1, 0, 1]);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        act(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(speed),
            },
        );
        settle(&mut g);
        start_attack(&mut g, speed);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        assert!(!g.legal_actions().contains(&Action::Block {
            blocker: bear,
            attacker: speed
        }));
        assert!(g.legal_actions().contains(&Action::Block {
            blocker: fast,
            attacker: speed
        }));
        let mut g = fresh(p);
        let bearer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let ray = put(&mut g, p, "Pym Particles", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
        cast(&mut g, ray, Some(Target::Card(bearer)));
        settle(&mut g);
        start_attack(&mut g, bearer);
        assert!(!g.object_view(bearer).tapped);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        assert!(!g.legal_actions().contains(&Action::Block {
            blocker,
            attacker: bearer
        }));
        for grown in [false, true] {
            let mut g = fresh(p);
            let stature = put(&mut g, p, "Stature, Size Shifter", Zone::Battlefield);
            let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if grown {
                mana(&mut g, p, [0, 2, 0, 0, 0, 1]);
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: stature,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                act(&mut g, Action::ChooseX { value: 1 });
                pay(&mut g);
                settle(&mut g);
            }
            start_attack(&mut g, stature);
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            assert_eq!(
                g.legal_actions().contains(&Action::Block {
                    blocker: bear,
                    attacker: stature
                }),
                grown
            );
        }
        let mut g = fresh(p);
        let ninja = put(&mut g, p, "Ninja of the Hand", Zone::Battlefield);
        let dinosaur = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
        start_attack(&mut g, ninja);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        act(
            &mut g,
            Action::Block {
                blocker: dinosaur,
                attacker: ninja,
            },
        );
        act(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.object_view(dinosaur).zone, Zone::Graveyard);
        assert_eq!(g.object_view(ninja).zone, Zone::Graveyard);
        let mut g = fresh(p);
        put(&mut g, p, "Storm, Windrider", Zone::Battlefield);
        let flyer = put(
            &mut g,
            p.other(),
            "S.H.I.E.L.D. Deployment Drone",
            Zone::Battlefield,
        );
        let ground = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.active = p.other();
        g.state.priority = p.other();
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        assert!(!g.legal_actions().contains(&Action::Attack { card: flyer }));
        assert!(g.legal_actions().contains(&Action::Attack { card: ground }));
        let mut g = fresh(p);
        put(&mut g, p, "Storm, Windrider", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let flyer = put(
            &mut g,
            p.other(),
            "S.H.I.E.L.D. Deployment Drone",
            Zone::Battlefield,
        );
        let ground = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        start_attack(&mut g, bear);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        assert!(!g.legal_actions().contains(&Action::Block {
            blocker: flyer,
            attacker: bear
        }));
        assert!(g.legal_actions().contains(&Action::Block {
            blocker: ground,
            attacker: bear
        }));
    }
}

#[test]
fn msh_c_any_caster_speedball_pump_and_optional_real_spell_retarget() {
    {
        let p = Player::P0;
        for caster in [p, p.other()] {
            let mut g = fresh(p);
            let speedball = put(&mut g, p, "Speedball, New Warrior", Zone::Battlefield);
            let dinosaur = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
            let shock = put(&mut g, caster, "Shock", Zone::Hand);
            mana(&mut g, caster, [0, 0, 0, 1, 0, 0]);
            if caster != p {
                act(&mut g, Action::Pass);
            }
            cast(&mut g, shock, Some(Target::Card(speedball)));
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Retarget { .. })
            });
            assert_eq!(stats(&g, speedball), (4, 4));
            act(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(dinosaur),
                },
            );
            act(&mut g, Action::FinishStackTargets);
            settle(&mut g);
            assert_eq!(g.object_view(dinosaur).damage, 2);
            assert_eq!(g.object_view(speedball).damage, 0);
            g.finish_cleanup();
            assert_eq!(stats(&g, speedball), (2, 2));
        }
    }
}

#[test]
fn msh_c_modal_faces_prowess_transformation_and_attached_controller_upkeep() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let monica = put(
            &mut g,
            p,
            "Monica Rambeau // Photon, Living Light",
            Zone::Battlefield,
        );
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let pym = put(&mut g, p, "Pym Particles", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
        cast(&mut g, pym, Some(Target::Card(bear)));
        settle(&mut g);
        assert_eq!(stats(&g, monica), (4, 4));
        assert_eq!(stats(&g, bear), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, monica), (3, 3));
        let mut g = fresh(p);
        let monica = put(
            &mut g,
            p,
            "Monica Rambeau // Photon, Living Light",
            Zone::Battlefield,
        );
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        mana(&mut g, p, [2, 0, 0, 1, 0, 2]);
        activate(&mut g, monica, 0, None);
        settle(&mut g);
        assert_eq!(stats(&g, monica), (4, 4));
        assert!(
            g.keywords(monica)
                .contains(Keywords::FLYING.union(Keywords::HEXPROOF))
        );
        let pym = put(&mut g, p, "Pym Particles", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
        cast(&mut g, pym, Some(Target::Card(bear)));
        assert!(matches!(g.state.pending, Pending::OrderTriggers { .. }));
        let first = g
            .state
            .ordering_triggers
            .iter()
            .find(|t| {
                t.source.is_some_and(|identity| identity.id == monica)
                    && t.kind == StackKind::TriggeredAbility { ability: 0 }
            })
            .unwrap()
            .id;
        act(&mut g, Action::PutTrigger { id: first });
        if matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            let second = g
                .state
                .ordering_triggers
                .iter()
                .find(|t| t.source.is_some_and(|identity| identity.id == monica))
                .unwrap()
                .id;
            act(&mut g, Action::PutTrigger { id: second });
        }
        settle(&mut g);
        assert_eq!(stats(&g, monica), (5, 5));
        assert_eq!(stats(&g, bear), (3, 3));
        g.finish_cleanup();
        assert_eq!(stats(&g, monica), (4, 4));
        assert_eq!(stats(&g, bear), (3, 3));
        let mut g = fresh(p);
        let card = put(
            &mut g,
            p,
            "Monica Rambeau // Photon, Living Light",
            Zone::Hand,
        );
        mana(&mut g, p, [2, 0, 0, 1, 0, 2]);
        act(&mut g, Action::AnnounceBack { card });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(stats(&g, card), (4, 4));
        assert!(g.keywords(card).contains(Keywords::HEXPROOF));
        let mut g = fresh(p);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, p, "Super Intelligence", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
        cast(&mut g, aura, Some(Target::Card(bear)));
        settle(&mut g);
        upkeep(&mut g, p);
        act(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 0);
        assert_eq!(g.hand_count(p.other()), 0);
        upkeep(&mut g, p.other());
        act(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.hand_count(p.other()), 1);
        let object = g.object_ref(bear);
        g.change_controller(object, p);
        upkeep(&mut g, p);
        act(&mut g, Action::Pass);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
    }
}

#[test]
fn msh_c_artifact_reduction_copy_targets_and_copy_activated_ability() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let shuri = put(&mut g, p, "Shuri, Wakandan Inventor", Zone::Battlefield);
        let from = put(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Battlefield);
        let to = put(&mut g, p, "S.H.I.E.L.D. Spy Kit", Zone::Battlefield);
        let drone = put(&mut g, p, "S.H.I.E.L.D. Deployment Drone", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 1]);
        cast(&mut g, drone, None);
        settle(&mut g);
        assert!(g.on_battlefield(drone));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(tokens(&g, p).len(), 1);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: shuri,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(to),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(from),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(to).name, "Mjölnir, Hammer of Thor");
        assert!(!g.object_view(to).is_legendary());
        assert!(g.on_battlefield(enemy));
        assert_eq!(g.object_view(enemy).damage, 0);
        g.finish_cleanup();
        assert_eq!(g.object_view(to).name, "S.H.I.E.L.D. Spy Kit");
        let mut g = fresh(p);
        let shuri = put(&mut g, p, "Shuri, Wakandan Inventor", Zone::Battlefield);
        let hammer = put(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Battlefield);
        mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: shuri,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(hammer),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(hammer),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(!g.object_view(hammer).is_legendary());
        let mut g = fresh(p);
        let scientist = put(&mut g, p, "Scientist Supreme of A.I.M.", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Panther Pounce", Zone::Hand);
        mana(&mut g, p, [1, 0, 0, 0, 0, 0]);
        announce(&mut g, spell, &[Target::Player(p), Target::Card(bear)]);
        settle(&mut g);
        let clue = tokens(&g, p)[0];
        mana(&mut g, p, [0, 0, 0, 0, 0, 2]);
        activate(&mut g, clue, 0, None);
        assert!(!g.on_battlefield(clue));
        let parent = g.state.stack.last().unwrap().id;
        g.state.active = p.other();
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}if *card==scientist)));
        g.state.active = p;
        activate(&mut g, scientist, 0, Some(Target::Stack(parent)));
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}if *card==scientist)));
        settle(&mut g);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.state.players[p.index()].life, 18);
    }
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let scientist = put(&mut g, p, "Scientist Supreme of A.I.M.", Zone::Battlefield);
        let dinosaur = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let hammer = put(&mut g, p, "Mjölnir, Hammer of Thor", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 1, 0, 3]);
        cast(&mut g, hammer, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(dinosaur),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let original = g.state.stack.last().unwrap().id;
        activate(&mut g, scientist, 0, Some(Target::Stack(original)));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Retarget { .. })
        });
        act(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(bear),
            },
        );
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(dinosaur).damage, 4);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 18);
        g.state.active = p.other();
        g.state.priority = p;
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}if *card==scientist)));
    }
}

#[test]
fn msh_c_linked_exile_copy_ward_and_continuous_control_untap_lock() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let dinosaur = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
        let aura = put(&mut g, p, "Secret Invasion", Zone::Hand);
        mana(&mut g, p, [0, 2, 0, 0, 0, 1]);
        cast(&mut g, aura, Some(Target::Card(bear)));
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(bear)
        }));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(dinosaur),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(dinosaur).zone, Zone::Exile);
        assert_eq!(stats(&g, bear), (7, 6));
        let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
        mana(&mut g, p.other(), [0, 1, 0, 0, 0, 2]);
        act(&mut g, Action::Pass);
        cast(&mut g, bounce, Some(Target::Card(bear)));
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        assert!(g.legal_actions().contains(&Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2]
        }));
        act(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert!(g.on_battlefield(bear));
        let remove = put(&mut g, p, "Disenchant", Zone::Hand);
        mana(&mut g, p, [1, 0, 0, 0, 0, 1]);
        cast(&mut g, remove, Some(Target::Card(aura)));
        settle(&mut g);
        assert_eq!(stats(&g, bear), (2, 2));
        assert!(g.on_battlefield(dinosaur));
        assert_eq!(g.object_view(dinosaur).controller, p.other());
        let mut g = fresh(p);
        let enemy = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
        let spider = put(&mut g, p, "Spider-Woman, Secret Agent", Zone::Hand);
        mana(&mut g, p, [1, 0, 0, 0, 0, 3]);
        cast(&mut g, spider, None);
        until(&mut g, |g| !matches!(g.state.pending, Pending::Priority));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        settle(&mut g);
        assert!(g.object_view(enemy).tapped);
        for locked in [true, false] {
            if !locked {
                let object = g.object_ref(spider);
                g.change_controller(object, p.other());
            }
            let pounce = put(&mut g, p, "Panther Pounce", Zone::Hand);
            mana(&mut g, p, [1, 0, 0, 0, 0, 0]);
            announce(&mut g, pounce, &[Target::Player(p), Target::Card(enemy)]);
            settle(&mut g);
            assert_eq!(g.object_view(enemy).tapped, locked);
        }
    }
}

#[test]
fn msh_c_plan_thresholds_grouped_events_and_all_three_saga_chapters() {
    {
        let p = Player::P0;
        let mut g = fresh(p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let plan = put(&mut g, p, "Political Triumph", Zone::Hand);
        mana(&mut g, p, [1, 0, 0, 0, 0, 0]);
        cast(&mut g, plan, None);
        settle(&mut g);
        let mut recruits = Vec::new();
        for count in 1..=4 {
            let recruit = put(&mut g, p, "Serpent Specialist", Zone::Hand);
            recruits.push(recruit);
            mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
            cast(&mut g, recruit, None);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectLibrary { .. })
            });
            let top = *g.state.players[p.index()].library.last().unwrap();
            act(
                &mut g,
                Action::ArrangeLibrary {
                    card: top,
                    destination: LibraryDestination::Top,
                },
            );
            settle(&mut g);
            if count < 4 {
                assert_eq!(g.counter_count(plan, CounterKind::Plan), count);
                assert!(g.on_battlefield(plan));
            }
        }
        assert_eq!(g.object_view(plan).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), 1);
        for id in recruits {
            assert_eq!(stats(&g, id), (2, 2));
        }
        assert_eq!(stats(&g, enemy), (2, 2));
        let mut g = fresh(p);
        let plan = put(&mut g, p, "Robot Domination", Zone::Hand);
        mana(&mut g, p, [0, 0, 1, 0, 0, 3]);
        cast(&mut g, plan, None);
        settle(&mut g);
        let rick = put(
            &mut g,
            p,
            "Rick Jones, Destined Sidekick",
            Zone::Battlefield,
        );
        for name in [
            "Serpent Specialist",
            "Savage Land Dinosaur",
            "Ninja of the Hand",
            "Forest",
        ] {
            put(&mut g, p, name, Zone::Library);
        }
        mana(&mut g, p, [0, 0, 0, 0, 0, 3]);
        activate(&mut g, rick, 0, None);
        settle(&mut g);
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 1);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.hand_count(p), 1);
        for expected_life in [18, 17] {
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            mana(&mut g, p, [0, 0, 0, 1, 0, 0]);
            cast(&mut g, shock, Some(Target::Card(bear)));
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, expected_life);
        }
        assert_eq!(g.object_view(plan).zone, Zone::Graveyard);
        let ts = tokens(&g, p);
        assert_eq!(ts.len(), 3);
        for t in ts {
            assert_eq!(stats(&g, t), (2, 2));
            assert!(g.has_subtype(t, "Robot") && g.has_subtype(t, "Villain"));
            assert!(g.current_kinds(t) & 16 != 0);
        }
        let mut g = fresh(p);
        let history = put(&mut g, p, "Rewrite History", Zone::Hand);
        mana(&mut g, p, [0, 1, 0, 0, 0, 2]);
        cast(&mut g, history, None);
        settle(&mut g);
        g.add_counter(history, CounterKind::Plan, 3);
        let shock = put(&mut g, p, "Shock", Zone::Graveyard);
        let negate = put(&mut g, p, "Negate", Zone::Graveyard);
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        let ship = put(&mut g, p, "S.H.I.E.L.D. Helicarrier", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let pet = put(&mut g, p, "Pet Avengers", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ship,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for id in [bear, pet] {
            let object = g.object_ref(id);
            act(&mut g, Action::ChooseCrewer { object });
        }
        act(&mut g, Action::FinishCrew);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        act(&mut g, Action::Discard { card: discard });
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        assert_eq!(g.object_view(history).zone, Zone::Graveyard);
        assert!(g.legal_actions().contains(&Action::FinishTargets));
        for id in [shock, negate] {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Hand);
        assert_eq!(g.object_view(negate).zone, Zone::Hand);
        assert_eq!(g.hand_count(p), 3);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        for enter_hero in [false, true] {
            let mut g = fresh(p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let hero = put(&mut g, p, "Patriot, Shield Wielder", Zone::Hand);
            let not_hero = put(&mut g, p, "Bear Cub", Zone::Hand);
            let expensive = put(&mut g, p, "Mister Fantastic, Reed Richards", Zone::Hand);
            let saga = put(&mut g, p, "Origin of the Avengers", Zone::Hand);
            mana(&mut g, p, [1, 0, 0, 0, 0, 1]);
            cast(&mut g, saga, None);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectLibrary { .. })
            });
            let top_two: Vec<_> = g.state.players[p.index()]
                .library
                .iter()
                .rev()
                .take(2)
                .copied()
                .collect();
            for card in top_two {
                act(
                    &mut g,
                    Action::ArrangeLibrary {
                        card,
                        destination: LibraryDestination::Top,
                    },
                );
            }
            settle(&mut g);
            assert_eq!(g.counter_count(saga, CounterKind::Lore), 1);
            g.state.turn += 1;
            g.begin_turn();
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            let before = g.hand_count(p);
            for id in [not_hero, expensive] {
                assert!(!g.legal_actions().contains(&Action::PutHandPermanent {
                    object: g.object_ref(id)
                }));
            }
            if enter_hero {
                let object = g.object_ref(hero);
                act(&mut g, Action::PutHandPermanent { object });
            } else {
                act(&mut g, Action::DeclineHandEntry);
            }
            settle(&mut g);
            assert_eq!(
                g.hand_count(p),
                if enter_hero { before - 1 } else { before + 1 }
            );
            assert_eq!(g.on_battlefield(hero), enter_hero);
            g.state.turn += 1;
            g.begin_turn();
            until(&mut g, |g| {
                g.state.step == Step::Main1 && g.state.pending == Pending::Priority
            });
            assert!(g.on_battlefield(saga));
            settle(&mut g);
            assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
            assert_eq!(stats(&g, bear), (3, 3));
            assert_eq!(stats(&g, enemy), (2, 2));
            if enter_hero {
                assert_eq!(stats(&g, hero), (3, 3));
            }
        }
    }
}

#[test]
fn msh_c_teamwork_modes_controller_damage_and_chosen_x_spell_copies() {
    {
        let p = Player::P0;
        for mode in [0, 1, 2] {
            let mut g = fresh(p);
            let teammate = put(&mut g, p, "Pet Avengers", Zone::Battlefield);
            let creature = put(
                &mut g,
                p.other(),
                "Mister Fantastic, Reed Richards",
                Zone::Battlefield,
            );
            let small = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let enchantment = put(&mut g, p.other(), "Robot Domination", Zone::Battlefield);
            let cheap = put(&mut g, p.other(), "Political Triumph", Zone::Battlefield);
            let spell = put(&mut g, p, "Murdock's Crusade", Zone::Hand);
            mana(&mut g, p, [1, 0, 0, 0, 0, 1]);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseTeamwork { pay: mode == 2 });
            if mode != 2 {
                assert!(!g.legal_actions().contains(&Action::ChooseMode { mode: 2 }));
            }
            act(&mut g, Action::ChooseMode { mode });
            if mode != 1 {
                assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(small)
                }));
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(creature),
                    },
                );
            }
            if mode != 0 {
                assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(cheap)
                }));
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enchantment),
                    },
                );
            }
            pay(&mut g);
            if mode == 2 {
                choose(&mut g, |a| matches!(a, Action::ChooseCastCost { .. }));
                let object = g.object_ref(teammate);
                act(&mut g, Action::ChooseTeammate { object });
                act(&mut g, Action::FinishTeamwork);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(creature).zone,
                if mode != 1 {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.object_view(enchantment).zone,
                if mode != 0 {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(teammate).tapped, mode == 2);
        }
        for teamwork in [false, true] {
            let mut g = fresh(p);
            let teammate = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
            let blast = put(&mut g, p, "Repulsor Blast", Zone::Hand);
            mana(&mut g, p, [0, 0, 0, 1, 0, 3]);
            act(&mut g, Action::AnnounceCast { card: blast });
            act(&mut g, Action::ChooseTeamwork { pay: teamwork });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            pay(&mut g);
            if teamwork {
                choose(&mut g, |a| matches!(a, Action::ChooseCastCost { .. }));
                let object = g.object_ref(teammate);
                act(&mut g, Action::ChooseTeammate { object });
                act(&mut g, Action::FinishTeamwork);
            }
            settle(&mut g);
            assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(enemy).damage,
                5,
                "Repulsor Blast deals exactly five in either branch"
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if teamwork { 18 } else { 20 }
            );
        }
        for (x, counter_original) in [(0, false), (2, false), (1, true)] {
            let mut g = fresh(p);
            let dinosaur = put(&mut g, p.other(), "Savage Land Dinosaur", Zone::Battlefield);
            let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let photon = put(&mut g, p, "Photon Blast Barrage", Zone::Hand);
            mana(&mut g, p, [0, 0, 0, 2, 0, x]);
            act(&mut g, Action::AnnounceCast { card: photon });
            act(&mut g, Action::ChooseX { value: x });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(dinosaur),
                },
            );
            pay(&mut g);
            if counter_original {
                let negate = put(&mut g, p, "Negate", Zone::Hand);
                mana(&mut g, p, [0, 1, 0, 0, 0, 1]);
                cast(&mut g, negate, Some(Target::Spell(photon)));
            }
            for i in 0..x {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::Retarget { .. })
                });
                if x == 2 && i == 0 {
                    act(
                        &mut g,
                        Action::SetStackTarget {
                            index: 0,
                            target: Target::Card(bear),
                        },
                    );
                }
                act(&mut g, Action::FinishStackTargets);
            }
            settle(&mut g);
            assert_eq!(g.object_view(dinosaur).damage, if x == 2 { 2 } else { 1 });
            assert_eq!(g.object_view(bear).damage, if x == 2 { 1 } else { 0 });
            assert_eq!(
                g.state.players[p.index()].spells_cast_this_turn,
                if counter_original { 2 } else { 1 }
            );
        }
    }
}
