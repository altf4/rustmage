//! Whole-card Power-up consumers: paid costs, exact targets, token rules and owned replay.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn announce(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn reject(g: &mut Game, action: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::ChooseActivationCost { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished payment");
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    finish(g);
    settle(g);
    id
}
fn choose_activation_cost(g: &mut Game) {
    if let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseActivationCost { .. }))
    {
        checked(g, a);
    }
}
fn activate(g: &mut Game, id: CardId, target: Option<CardId>) {
    checked(g, announce(id));
    choose_activation_cost(g);
    if let Some(card) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    finish(g);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..90 {
        if stop(g) {
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
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn attackers(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
}
fn token_ids(g: &Game, p: Player, recipe: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == recipe && o.is_token())
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn msh_power_cards_paid_entry_costs_counters_keywords_and_lifetime() {
    let rows = [
        ("Bold Biochemist", 4, 1, 1, 3, Keywords::NONE),
        ("Brave Brawler", 3, 2, 2, 1, Keywords::LIFELINK),
        ("Ninja of the Hand", 2, 1, 2, 2, Keywords::DEATHTOUCH),
        ("Pet Avengers", 3, 1, 4, 4, Keywords::REACH),
        ("Serpent Specialist", 3, 2, 1, 1, Keywords::DEATHTOUCH),
        ("Ultron Drone", 3, 2, 2, 3, Keywords::NONE),
        ("Viv Vision, Teen Synthezoid", 4, 2, 2, 2, Keywords::FLYING),
        ("Volcanic Villain", 3, 2, 3, 2, Keywords::HASTE),
        ("White Tiger, Ava Ayala", 4, 1, 2, 2, Keywords::NONE),
    ];
    for p in [Player::P0, Player::P1] {
        for (name, cost, counters, power, toughness, keywords) in rows {
            let mut g = setup(p);
            let id = paid(&mut g, p, name);
            let hand = g.hand_count(p);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, cost - 1];
            reject(&mut g, announce(id));
            g.state.players[p.index()].mana[5] = cost;
            activate(&mut g, id, None);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), counters);
            assert_eq!(
                (g.object_view(id).power, g.object_view(id).toughness),
                (
                    power + i128::from(counters),
                    toughness + i128::from(counters)
                )
            );
            assert!(g.object_view(id).keywords.contains(keywords));
            if name == "Bold Biochemist" {
                assert_eq!(g.hand_count(p), hand + 2);
            }
            g.state.players[p.index()].mana = [30; 6];
            reject(&mut g, announce(id));
            assert_eq!(g.object_view(id).power_up_uses[0].count, 1);
        }
    }
}

#[test]
fn msh_power_cards_biochemist_draw_survives_blink_and_keeps_hand_private() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Bold Biochemist");
        let secret = named(&mut g, p, "Hill Giant", Zone::Library);
        let hand = g.hand_count(p);
        activate(&mut g, id, None);
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 2);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        assert!(g.object_view(id).power_up_uses.is_empty());
        assert!(!g.observe(p.other()).objects.iter().any(|v| v.id == secret));
    }
}

#[test]
fn msh_power_cards_ninja_discard_is_opponents_private_choice_even_after_departure() {
    for p in [Player::P0, Player::P1] {
        for departed in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Ninja of the Hand");
            let own = named(&mut g, p, "Hill Giant", Zone::Hand);
            let discarded = named(&mut g, p.other(), "Forest", Zone::Hand);
            activate(&mut g, id, None);
            if departed {
                g.move_object(id, Zone::Exile);
            }
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            reject(&mut g, Action::Discard { card: own });
            assert!(!g.observe(p).objects.iter().any(|o| o.id == discarded));
            checked(&mut g, Action::Discard { card: discarded });
            settle(&mut g);
            assert_eq!(g.object_view(discarded).zone, Zone::Graveyard);
            assert_eq!(g.object_view(own).zone, Zone::Hand);
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                u64::from(!departed)
            );
        }
    }
}

#[test]
fn msh_power_cards_abomination_hybrid_cast_and_reduction_optional_fight() {
    for p in [Player::P0, Player::P1] {
        for color in [3, 4] {
            for targeted in [false, true] {
                let mut g = setup(p);
                let id = named(&mut g, p, "Abomination, Terrifying Titan", Zone::Hand);
                g.state.players[p.index()].mana = [0; 6];
                g.state.players[p.index()].mana[color] = 4;
                checked(&mut g, Action::AnnounceCast { card: id });
                finish(&mut g);
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                // Entry reduction {3}{R/G} leaves {2}{R/G}, funded entirely by either half.
                g.state.players[p.index()].mana = [0; 6];
                g.state.players[p.index()].mana[color] = 3;
                checked(&mut g, announce(id));
                choose_activation_cost(&mut g);
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
                if targeted {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(enemy),
                        },
                    );
                }
                finish(&mut g);
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(g.object_view(id).power, 5);
                assert_eq!(g.object_view(id).damage, if targeted { 3 } else { 0 });
                assert_eq!(
                    g.object_view(enemy).zone,
                    if targeted {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                assert!(g.object_view(id).keywords.contains(Keywords::TRAMPLE));
            }
        }
    }
}

#[test]
fn msh_power_cards_abomination_invalid_target_counters_whole_packet_and_departed_source_cannot_fight()
 {
    for p in [Player::P0, Player::P1] {
        for departed in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Abomination, Terrifying Titan");
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            activate(&mut g, id, Some(enemy));
            if departed {
                g.move_object(id, Zone::Exile);
            } else {
                g.move_object(enemy, Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
            assert_eq!(g.object_view(enemy).damage, 0);
        }
    }
}

#[test]
fn msh_power_cards_she_hulk_optional_artifact_enchantment_target_and_invalid_packet() {
    for p in [Player::P0, Player::P1] {
        for choice in 0..4 {
            let mut g = setup(p);
            let id = paid(&mut g, p, "She-Hulk, Jade Defender");
            let artifact = named(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
            let enchant = named(&mut g, p, "Training Regimen", Zone::Battlefield);
            let creature = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let target = if choice == 1 {
                Some(artifact)
            } else if choice >= 2 {
                Some(enchant)
            } else {
                None
            };
            // {4}{G}{G} less entry cost {3}{G} leaves {1}{G}.
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            checked(&mut g, announce(id));
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            if let Some(card) = target {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(card),
                    },
                );
            }
            finish(&mut g);
            if choice == 3 {
                g.move_object(enchant, Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                u64::from(choice != 3)
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if let Some(card) = target {
                assert_eq!(
                    g.object_view(card).zone,
                    if choice == 3 {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            assert!(
                g.object_view(id)
                    .keywords
                    .contains(Keywords::REACH.union(Keywords::TRAMPLE))
            );
        }
    }
}

#[test]
fn msh_power_cards_unliving_optional_own_creature_return_public_identity_and_invalid_packet() {
    for p in [Player::P0, Player::P1] {
        for choice in 0..3 {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Unliving Legionnaire");
            let own = named(&mut g, p, "Hill Giant", Zone::Graveyard);
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
            let land = named(&mut g, p, "Forest", Zone::Graveyard);
            // {5}{B}{B} less {3}{B} leaves {2}{B}.
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
            checked(&mut g, announce(id));
            for card in [enemy, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(card),
                    },
                );
            }
            if choice != 0 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
            }
            finish(&mut g);
            if choice == 2 {
                g.move_object(own, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(id, CounterKind::PlusOne),
                if choice == 2 { 0 } else { 2 }
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.object_view(own).zone,
                match choice {
                    0 => Zone::Graveyard,
                    1 => Zone::Hand,
                    _ => Zone::Exile,
                }
            );
            if choice == 1 {
                assert!(g.observe(p.other()).objects.iter().any(|o| o.id == own));
            }
            assert!(g.object_view(id).keywords.contains(Keywords::FLYING));
        }
    }
}

#[test]
fn msh_power_cards_hercules_haste_vigilance_indestructible_expire_counters_remain() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Hercules, Prince of Power");
        attackers(&mut g, p);
        reject(&mut g, Action::Attack { card: id });
        g.state.pending = Pending::Priority;
        g.state.step = Step::Main1;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        activate(&mut g, id, None);
        settle(&mut g);
        assert!(
            g.object_view(id).keywords.contains(
                Keywords::VIGILANCE
                    .union(Keywords::HASTE)
                    .union(Keywords::INDESTRUCTIBLE)
            )
        );
        g.state.players[p.index()].mana = [30; 6];
        for _ in 0..2 {
            let bolt = named(&mut g, p, "Lightning Bolt", Zone::Hand);
            cast(&mut g, bolt, None, &[Target::Card(id)]);
            settle(&mut g);
        }
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert_eq!(g.object_view(id).damage, 6);
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: id });
        assert!(!g.object_view(id).tapped);
        checked(&mut g, Action::FinishAttackers);
        next_turn(&mut g);
        assert_eq!(g.object_view(id).power, 4);
        assert!(
            !g.object_view(id).keywords.contains(
                Keywords::INDESTRUCTIBLE
                    .union(Keywords::HASTE)
                    .union(Keywords::VIGILANCE)
            )
        );
    }
}

#[test]
fn msh_power_cards_pet_and_ultron_tokens_are_exact_recipes_and_trigger_villain_observer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let observer = paid(&mut g, p, "Crossbones, Malicious Mercenary");
        let pet = paid(&mut g, p, "Pet Avengers");
        activate(&mut g, pet, None);
        settle(&mut g);
        let hero = token_ids(
            &g,
            p,
            Card::from_name("@token/58de30178066df1fd0ce4539").unwrap(),
        )[0];
        let v = g.object_view(hero);
        assert_eq!((v.power, v.toughness, v.colors), (3, 2, 1));
        assert!(v.keywords.contains(Keywords::VIGILANCE));
        assert!(v.has_subtype("Hero"));
        assert!(!v.tapped);
        assert_eq!(g.counter_count(observer, CounterKind::PlusOne), 0);
        let drone = paid(&mut g, p, "Ultron Drone");
        let before = g.counter_count(observer, CounterKind::PlusOne);
        activate(&mut g, drone, None);
        g.move_object(drone, Zone::Exile);
        settle(&mut g);
        let robot = token_ids(&g, p, Card::MshRobotVillainToken)[0];
        let v = g.object_view(robot);
        assert_eq!((v.power, v.toughness, v.colors, v.kinds), (2, 2, 0, 17));
        assert!(v.has_subtype("Robot") && v.has_subtype("Villain"));
        assert!(!v.tapped);
        assert_eq!(
            g.counter_count(observer, CounterKind::PlusOne),
            before,
            "same-turn once limit remains consumed by Drone"
        );
        assert!(
            !Card::from_name("@token/c9fd6abc12b4083149908964")
                .unwrap()
                .has_subtype("Villain")
        );
        assert_eq!(Card::MshRobotVillainToken.info().deck_limit, None);
        g.state.objects[hero as usize].summoning_sick = false;
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: hero });
        assert!(!g.object_view(hero).tapped);
    }
}

#[test]
fn msh_power_cards_white_tiger_token_name_legend_rule_and_one_blocker() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Wonder Man, Hollywood Hero");
        let source = paid(&mut g, p, "White Tiger, Ava Ayala");
        activate(&mut g, source, None);
        settle(&mut g);
        let tiger = token_ids(&g, p, Card::MshTigerGodToken)[0];
        let v = g.object_view(tiger);
        assert_eq!(v.name(), "The Tiger God");
        assert_eq!((v.power, v.toughness, v.colors), (4, 4, 16));
        assert!(v.has_subtype("Cat") && v.has_subtype("God"));
        assert!(v.keywords.contains(Keywords::MAX_ONE_BLOCKER));
        assert_eq!(Card::MshTigerGodToken.info().deck_limit, None);
        activate(&mut g, source, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Legend { .. })
        });
        let object = g.object_ref(tiger);
        checked(&mut g, Action::KeepLegend { object });
        settle(&mut g);
        assert_eq!(token_ids(&g, p, Card::MshTigerGodToken), vec![tiger]);
        let first = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let second = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[tiger as usize].summoning_sick = false;
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: tiger });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: first,
                attacker: tiger,
            },
        );
        reject(
            &mut g,
            Action::Block {
                blocker: second,
                attacker: tiger,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}

#[test]
fn msh_power_cards_viv_attacks_check_resolution_power_live_and_departure_lki() {
    for p in [Player::P0, Player::P1] {
        for case in 0..5 {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Viv Vision, Teen Synthezoid");
            if case != 0 {
                activate(&mut g, id, None);
                settle(&mut g);
            }
            g.state.objects[id as usize].summoning_sick = false;
            let hand = g.hand_count(p);
            attackers(&mut g, p);
            checked(&mut g, Action::Attack { card: id });
            checked(&mut g, Action::FinishAttackers);
            assert!(
                g.state
                    .pending_triggers
                    .iter()
                    .chain(g.state.stack.iter())
                    .any(|s| s.card == id)
            );
            match case {
                0 => g.add_counter(id, CounterKind::PlusOne, 2),
                2 => g.set_counters(id, Counters::default()),
                3 => {
                    g.move_object(id, Zone::Exile);
                }
                4 => {
                    g.move_object(id, Zone::Exile);
                    g.move_object(id, Zone::Battlefield);
                }
                _ => {}
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + usize::from(case != 2));
        }
    }
}

#[test]
fn msh_power_cards_counter_boosted_lifelink_deathtouch_haste_flying_and_reach_combat() {
    for p in [Player::P0, Player::P1] {
        for (name, damage) in [("Brave Brawler", 4), ("Volcanic Villain", 5)] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name);
            activate(&mut g, id, None);
            settle(&mut g);
            if name == "Brave Brawler" {
                g.state.objects[id as usize].summoning_sick = false;
            }
            attackers(&mut g, p);
            checked(&mut g, Action::Attack { card: id });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| g.state.step == Step::Main2);
            assert_eq!(g.state.players[p.other().index()].life, 20 - damage);
            assert_eq!(
                g.state.players[p.index()].life,
                if name == "Brave Brawler" { 24 } else { 20 }
            );
        }
        let mut g = setup(p);
        let serpent = paid(&mut g, p, "Serpent Specialist");
        activate(&mut g, serpent, None);
        settle(&mut g);
        let turtle = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        g.state.objects[serpent as usize].summoning_sick = false;
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: serpent });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: turtle,
                attacker: serpent,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::Main2);
        assert_eq!(g.object_view(turtle).zone, Zone::Graveyard);
        assert_eq!(g.object_view(serpent).zone, Zone::Battlefield);
        let mut g = setup(p);
        let pet = paid(&mut g, p, "Pet Avengers");
        activate(&mut g, pet, None);
        settle(&mut g);
        let flyer = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        g.state.objects[flyer as usize].summoning_sick = false;
        attackers(&mut g, p.other());
        checked(&mut g, Action::Attack { card: flyer });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: pet,
                attacker: flyer,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| g.state.step == Step::Main2);
        assert_eq!(g.object_view(flyer).zone, Zone::Graveyard);
        assert_eq!(g.object_view(pet).zone, Zone::Battlefield);
        let mut g = setup(p);
        let viv = paid(&mut g, p, "Viv Vision, Teen Synthezoid");
        activate(&mut g, viv, None);
        settle(&mut g);
        let ground = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[viv as usize].summoning_sick = false;
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: viv });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: viv,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}
