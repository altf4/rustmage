//! Printed exhaust, Vehicle and green entry outcomes using the frozen shared engine.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn drain(g: &mut Game) {
    for _ in 0..96 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("bounded exhaust/entry resolution");
}
fn activate(g: &mut Game, card: CardId, ability: u16, target: Option<CardId>) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(id) = target {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
        }
    }
    pay(g);
    drain(g);
}
fn crew(g: &mut Game, card: CardId, ability: u16, contributors: &[CardId]) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
    for (i, &id) in contributors.iter().enumerate() {
        let object = g.object_ref(id);
        act(g, Action::ChooseCrewer { object });
        if i + 1 < contributors.len() {
            reject(g, Action::FinishCrew);
        }
    }
    act(g, Action::FinishCrew);
    drain(g);
}
fn tokens(g: &Game, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(id as CardId, subtype))
                .then_some(id as CardId)
        })
        .collect()
}
fn attack(g: &mut Game, card: CardId) {
    g.state.objects[card as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    act(g, Action::Attack { card });
    act(g, Action::FinishAttackers);
    drain(g);
}
#[test]
fn afterburner_and_aetherhive_observe_actual_exhaust_and_greenbelt_has_both_abilities() {
    let mut g = scenario(Player::P0);
    let expert = put(&mut g, Player::P0, "Afterburner Expert", Zone::Graveyard);
    let hive = put(&mut g, Player::P0, "Rangers' Aetherhive", Zone::Battlefield);
    let guardian = put(&mut g, Player::P0, "Greenbelt Guardian", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    activate(&mut g, guardian, 0, Some(enemy));
    assert!(g.keywords(enemy).contains(Keywords::TRAMPLE));
    assert_eq!(g.state.objects[expert as usize].zone, Zone::Graveyard);
    assert!(tokens(&g, "Thopter").is_empty());
    activate(&mut g, guardian, 1, None);
    assert_eq!(stats(&g, guardian), (5, 5));
    assert_eq!(g.state.objects[expert as usize].zone, Zone::Battlefield);
    assert_eq!(stats(&g, expert), (4, 2));
    activate(&mut g, expert, 0, None);
    assert_eq!(stats(&g, expert), (6, 4));
    let thopters = tokens(&g, "Thopter");
    assert_eq!(thopters.len(), 2);
    for id in &thopters {
        assert_eq!(stats(&g, *id), (1, 1));
        assert_eq!(g.colors(*id), 0);
        assert!(g.keywords(*id).contains(Keywords::FLYING));
        assert!(g.current_kinds(*id) & 16 != 0);
    }
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: expert,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: guardian,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    crew(&mut g, hive, 0, &[thopters[0]]);
    assert_eq!(stats(&g, hive), (3, 5));
    assert!(g.keywords(hive).contains(Keywords::VIGILANCE));
    g.finish_cleanup();
    assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
    assert!(!g.is_creature(hive));
}
#[test]
fn boom_scholar_discount_excludes_itself_and_trample_freezes_creature_vehicle_union() {
    let mut g = scenario(Player::P0);
    let scholar = put(&mut g, Player::P0, "Boom Scholar", Zone::Battlefield);
    let guardian = put(&mut g, Player::P0, "Greenbelt Guardian", Zone::Battlefield);
    let vehicle = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let boots = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    g.state.players[0].mana = [0, 0, 0, 0, 1, 1];
    activate(&mut g, guardian, 1, None);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    g.state.players[0].mana = [0, 0, 0, 1, 1, 2];
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: scholar,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    g.state.players[0].mana = [0, 0, 0, 1, 1, 4];
    activate(&mut g, scholar, 0, None);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(stats(&g, scholar), (5, 5));
    for id in [scholar, guardian, vehicle] {
        assert!(g.keywords(id).contains(Keywords::TRAMPLE));
    }
    assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
    assert!(!g.keywords(boots).contains(Keywords::TRAMPLE));
    assert!(!g.is_creature(vehicle));
    g.finish_cleanup();
    assert!(!g.keywords(vehicle).contains(Keywords::TRAMPLE));
    assert_eq!(g.counter_count(scholar, CounterKind::PlusOne), 2);
}
#[test]
fn elvish_refueler_reuses_only_one_spent_exhaust_before_this_turns_first_exhaust() {
    let mut g = scenario(Player::P0);
    let guardian = put(&mut g, Player::P0, "Greenbelt Guardian", Zone::Battlefield);
    activate(&mut g, guardian, 1, None);
    let refueler = put(&mut g, Player::P0, "Elvish Refueler", Zone::Battlefield);
    let spent = Action::AnnounceActivate {
        card: guardian,
        ability: 1,
        counter_payment: Counters::default(),
    };
    reject(&mut g, spent.clone());
    g.finish_cleanup();
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.priority = Player::P0;
    g.state.players[0].mana = [8; 6];
    reject(&mut g, spent.clone());
    g.finish_cleanup();
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.priority = Player::P0;
    g.state.players[0].mana = [8; 6];
    assert!(g.legal_actions().contains(&spent));
    activate(&mut g, guardian, 1, None);
    assert_eq!(g.counter_count(guardian, CounterKind::PlusOne), 6);
    reject(&mut g, spent);
    activate(&mut g, refueler, 0, None);
    assert_eq!(stats(&g, refueler), (3, 4));
}
#[test]
fn earthrumbler_pays_a_filtered_graveyard_exile_or_exact_crew_three_and_animation_expires() {
    for exile in [true, false] {
        let mut g = scenario(Player::P0);
        let vehicle = put(&mut g, Player::P0, "Earthrumbler", Zone::Hand);
        cast(&mut g, vehicle, None);
        drain(&mut g);
        assert!(!g.is_creature(vehicle));
        if exile {
            let bad = put(&mut g, Player::P0, "Opt", Zone::Graveyard);
            let good = put(&mut g, Player::P0, "Gleaming Barrier", Zone::Graveyard);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: vehicle,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let bad = g.object_ref(bad);
            reject(&mut g, Action::ChooseAdditionalCost { object: bad });
            let object = g.object_ref(good);
            act(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            drain(&mut g);
            assert_eq!(g.state.objects[good as usize].zone, Zone::Exile);
        } else {
            let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
            let elf = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
            crew(&mut g, vehicle, 1, &[bear, elf]);
        }
        assert_eq!(stats(&g, vehicle), (7, 6));
        assert!(
            g.keywords(vehicle)
                .contains(Keywords::TRAMPLE.union(Keywords::VIGILANCE))
        );
        g.finish_cleanup();
        assert!(!g.is_creature(vehicle));
    }
}
#[test]
fn rocketeer_exhaust_is_permanent_crew_is_temporary_and_attack_treasure_is_usable() {
    let mut g = scenario(Player::P0);
    let buggy = put(&mut g, Player::P0, "Rocketeer Boostbuggy", Zone::Hand);
    cast(&mut g, buggy, None);
    drain(&mut g);
    let pilot = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    crew(&mut g, buggy, 1, &[pilot]);
    assert_eq!(stats(&g, buggy), (3, 2));
    g.finish_cleanup();
    assert!(!g.is_creature(buggy));
    g.state.step = Step::Main1;
    g.state.active = Player::P0;
    g.state.priority = Player::P0;
    g.state.pending = Pending::Priority;
    g.state.players[0].mana = [8; 6];
    activate(&mut g, buggy, 0, None);
    assert_eq!(stats(&g, buggy), (4, 3));
    attack(&mut g, buggy);
    let treasure = tokens(&g, "Treasure");
    assert_eq!(treasure.len(), 1);
    let green = [0, 0, 0, 0, 1, 0];
    choose(
        &mut g,
        |a| matches!(a,Action::ActivateMana{card,production,..} if *card==treasure[0] && *production==green),
    );
    assert_eq!(g.state.objects[treasure[0] as usize].zone, Zone::Outside);
    g.finish_cleanup();
    assert!(g.is_creature(buggy));
    assert_eq!(stats(&g, buggy), (4, 3));
}
#[test]
fn ooze_patrol_counts_entire_graveyard_after_two_mills_and_deduplicates_artifact_creature() {
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Gleaming Barrier", Zone::Graveyard);
    put(&mut g, Player::P0, "Opt", Zone::Graveyard);
    let artifact = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Library);
    let creature = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
    let ooze = put(&mut g, Player::P0, "Ooze Patrol", Zone::Hand);
    cast(&mut g, ooze, None);
    drain(&mut g);
    for id in [artifact, creature] {
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
    }
    assert_eq!(g.counter_count(ooze, CounterKind::PlusOne), 3);
    assert_eq!(stats(&g, ooze), (5, 5));
}
#[test]
fn fang_guardian_flashes_in_and_targets_only_another_controlled_creature_or_vehicle() {
    for vehicle in [true, false] {
        let mut g = scenario(Player::P0);
        let recipient = put(
            &mut g,
            Player::P0,
            if vehicle {
                "Cultivator's Caravan"
            } else {
                "Grizzly Bears"
            },
            Zone::Battlefield,
        );
        let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let boots = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
        g.state.active = Player::P1;
        g.state.step = Step::Upkeep;
        g.state.priority = Player::P0;
        let fang = put(&mut g, Player::P0, "Fang Guardian", Zone::Hand);
        cast(&mut g, fang, None);
        for _ in 0..24 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(recipient),
            }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        for bad in [fang, enemy, boots] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(recipient),
            },
        );
        drain(&mut g);
        assert_eq!(stats(&g, fang), (4, 2));
        assert!(g.keywords(fang).contains(Keywords::FLASH));
        if vehicle {
            crew(&mut g, recipient, 1, &[fang]);
            assert_eq!(stats(&g, recipient), (7, 7));
        } else {
            assert_eq!(stats(&g, recipient), (4, 4));
        }
        g.finish_cleanup();
        if vehicle {
            assert!(!g.is_creature(recipient));
        } else {
            assert_eq!(stats(&g, recipient), (2, 2));
        }
    }
}
#[test]
fn district_mascot_entry_and_saddled_attack_fund_two_counter_removal_even_if_source_dies() {
    let mut g = scenario(Player::P0);
    let dog = put(&mut g, Player::P0, "District Mascot", Zone::Hand);
    cast(&mut g, dog, None);
    drain(&mut g);
    assert_eq!(stats(&g, dog), (1, 1));
    let artifact = put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    let remove = Counters::of(CounterKind::PlusOne, 2);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: dog,
            ability: 0,
            counter_payment: remove.clone(),
        },
    );
    let pilot = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: dog,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    let object = g.object_ref(pilot);
    act(&mut g, Action::ChooseSaddler { object });
    act(&mut g, Action::FinishSaddle);
    drain(&mut g);
    attack(&mut g, dog);
    assert_eq!(g.counter_count(dog, CounterKind::PlusOne), 2);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: dog,
            ability: 0,
            counter_payment: remove,
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.objects[dog as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[artifact as usize].zone, Zone::Graveyard);
}
