//! Printed Mount/Vehicle abilities wired to the accepted engine baseline.
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
    panic!("bounded Mount/Vehicle resolution");
}
fn reach(g: &mut Game, wanted: impl Fn(&Action) -> bool) {
    for _ in 0..96 {
        if g.legal_actions().iter().any(&wanted) {
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
    panic!("bounded Mount/Vehicle decision");
}
fn activate(g: &mut Game, card: CardId, ability: u16) {
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
}
fn crew(g: &mut Game, card: CardId, ability: u16, contributors: &[CardId]) {
    activate(g, card, ability);
    for &id in contributors {
        let object = g.object_ref(id);
        act(g, Action::ChooseCrewer { object });
    }
    act(g, Action::FinishCrew);
    drain(g);
}
#[test]
fn every_vehicle_pays_its_printed_crew_threshold_and_uses_its_printed_stats() {
    for (name, ability, minimum, expected, keywords) in [
        ("Apocalypse Runner", 1, 3, (6, 5), Keywords::NONE),
        ("Debris Beetle", 0, 2, (6, 6), Keywords::TRAMPLE),
        ("Lumbering Worldwagon", 0, 4, (4, 4), Keywords::NONE),
        ("Veloheart Bike", 1, 2, (4, 2), Keywords::NONE),
        ("Thunderous Velocipede", 0, 3, (5, 5), Keywords::TRAMPLE),
        (
            "Thundering Broodwagon",
            0,
            3,
            (6, 5),
            Keywords::REACH.union(Keywords::MENACE),
        ),
    ] {
        let mut g = scenario(Player::P0);
        let vehicle = put(&mut g, Player::P0, name, Zone::Battlefield);
        for _ in 0..4 {
            put(&mut g, Player::P0, "Forest", Zone::Battlefield);
        }
        let first = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        let second = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        if minimum > 2 {
            g.add_counter(second, CounterKind::PlusOne, minimum - 2);
        }
        assert!(!g.is_creature(vehicle), "{name}");
        activate(&mut g, vehicle, ability);
        let object = g.object_ref(first);
        act(&mut g, Action::ChooseCrewer { object });
        reject(&mut g, Action::FinishCrew);
        let object = g.object_ref(second);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        drain(&mut g);
        assert_eq!(stats(&g, vehicle), expected, "{name}");
        assert!(g.keywords(vehicle).contains(keywords), "{name}");
        assert!(g.object_view(first).tapped && g.object_view(second).tapped);
        g.finish_cleanup();
        assert!(!g.is_creature(vehicle), "{name} crew expires");
    }
}
#[test]
fn debris_beetle_entry_drains_three_and_bike_entry_gains_two_before_any_crewing() {
    let mut g = scenario(Player::P0);
    let beetle = put(&mut g, Player::P0, "Debris Beetle", Zone::Hand);
    cast(&mut g, beetle, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 23);
    assert_eq!(g.state.players[1].life, 17);
    let bike = put(&mut g, Player::P0, "Veloheart Bike", Zone::Hand);
    cast(&mut g, bike, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 25);
    assert!(!g.is_creature(bike));
    g.state.players[0].mana = [0; 6];
    let production = [0, 1, 0, 0, 0, 0];
    act(
        &mut g,
        Action::ActivateMana {
            card: bike,
            ability: 0,
            counter_payment: Counters::default(),
            payment: [0; 6],
            production,
        },
    );
    assert_eq!(g.state.players[0].mana, production);
    assert!(g.object_view(bike).tapped);
}
#[test]
fn thunderous_velocipede_adds_one_or_three_on_actual_entry_without_counting_itself_or_artifacts() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Thunderous Velocipede", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
    for (name, expected) in [
        ("Hill Giant", 1),
        ("Serra Angel", 3),
        ("Cultivator's Caravan", 1),
        ("Swiftfoot Boots", 0),
    ] {
        let entrant = put(&mut g, Player::P0, name, Zone::Hand);
        cast(&mut g, entrant, None);
        drain(&mut g);
        assert_eq!(
            g.counter_count(entrant, CounterKind::PlusOne),
            expected,
            "{name}"
        );
    }
    g.state.active = Player::P1;
    g.state.priority = Player::P1;
    let opponent = put(&mut g, Player::P1, "Hill Giant", Zone::Hand);
    cast(&mut g, opponent, None);
    drain(&mut g);
    assert_eq!(g.counter_count(opponent, CounterKind::PlusOne), 0);
}
#[test]
fn thundering_broodwagon_destroys_only_opposing_small_nonland_or_cycles_without_entry_effect() {
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    let own = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let large = put(&mut g, Player::P1, "Serra Angel", Zone::Battlefield);
    let land = put(&mut g, Player::P1, "Forest", Zone::Battlefield);
    let source = put(&mut g, Player::P0, "Thundering Broodwagon", Zone::Hand);
    cast(&mut g, source, None);
    reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
    for bad in [own, large, land] {
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
            target: Target::Card(target),
        },
    );
    drain(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    let cycling = put(&mut g, Player::P0, "Thundering Broodwagon", Zone::Hand);
    activate(&mut g, cycling, 1);
    drain(&mut g);
    assert_eq!(g.object_view(cycling).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.object_view(own).zone, Zone::Battlefield);
}
#[test]
fn apocalypse_runner_rejects_large_or_foreign_targets_and_its_grants_expire() {
    let mut g = scenario(Player::P0);
    let runner = put(&mut g, Player::P0, "Apocalypse Runner", Zone::Battlefield);
    let small = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let large = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: runner,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for bad in [large, enemy] {
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
            target: Target::Card(small),
        },
    );
    pay(&mut g);
    drain(&mut g);
    assert!(g.object_view(runner).tapped);
    assert!(
        g.keywords(small)
            .contains(Keywords::LIFELINK.union(Keywords::CANT_BE_BLOCKED))
    );
    g.finish_cleanup();
    assert!(
        !g.keywords(small)
            .intersects(Keywords::LIFELINK.union(Keywords::CANT_BE_BLOCKED))
    );
}
#[test]
fn loxodon_surveyor_starts_engines_and_its_graveyard_draw_requires_its_owners_max_speed() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Loxodon Surveyor", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].speed, 1);
    assert_eq!(stats(&g, source), (3, 3));
    g.move_object(source, Zone::Graveyard);
    g.state.objects[source as usize].controller = Player::P1;
    let action = Action::AnnounceSourceGrant {
        source: g.object_ref(source),
        ability: 0,
    };
    reject(&mut g, action.clone());
    g.state.players[0].speed = 4;
    act(&mut g, action);
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Exile);
    drain(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}
#[test]
fn stampeding_scurryfoot_exhaust_adds_a_counter_and_an_actual_three_three_elephant_once() {
    let mut g = scenario(Player::P0);
    let source = put(
        &mut g,
        Player::P0,
        "Stampeding Scurryfoot",
        Zone::Battlefield,
    );
    activate(&mut g, source, 0);
    drain(&mut g);
    assert_eq!(stats(&g, source), (2, 2));
    let elephant = g
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(id as CardId, "Elephant"))
                .then_some(id as CardId)
        })
        .unwrap();
    assert_eq!(stats(&g, elephant), (3, 3));
    assert_eq!(g.object_view(elephant).colors, 16);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn regal_imperiosaur_buffs_only_other_controlled_dinosaurs_and_updates_when_control_changes() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Regal Imperiosaur", Zone::Battlefield);
    let own = put(&mut g, Player::P0, "Agonasaur Rex", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Agonasaur Rex", Zone::Battlefield);
    let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&g, source), (5, 4));
    assert_eq!(stats(&g, own), (9, 9));
    assert_eq!(stats(&g, enemy), (8, 8));
    assert_eq!(stats(&g, bear), (2, 2));
    g.state.objects[own as usize].controller = Player::P1;
    assert_eq!(stats(&g, own), (8, 8));
}
#[test]
fn veteran_beastrider_grows_its_controller_only_and_untaps_at_its_own_end_step() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Veteran Beastrider", Zone::Battlefield);
    let own = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    activate(&mut g, source, 0);
    drain(&mut g);
    assert_eq!(stats(&g, source), (4, 5));
    assert_eq!(stats(&g, own), (3, 3));
    assert_eq!(stats(&g, enemy), (2, 2));
    for id in [source, own, enemy] {
        g.state.objects[id as usize].tapped = true;
    }
    g.state.active = Player::P1;
    g.state.priority = Player::P1;
    g.begin_step(Step::End);
    g.prepare_triggers();
    drain(&mut g);
    assert!(g.object_view(source).tapped && g.object_view(own).tapped);
    g.state.active = Player::P0;
    g.state.priority = Player::P0;
    g.begin_step(Step::End);
    g.prepare_triggers();
    drain(&mut g);
    assert!(!g.object_view(source).tapped && !g.object_view(own).tapped);
    assert!(g.object_view(enemy).tapped);
    g.finish_cleanup();
    assert_eq!(stats(&g, own), (2, 2));
}
#[test]
fn beastrider_vanguard_looks_privately_takes_a_permanent_and_orders_the_other_cards() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Beastrider Vanguard", Zone::Battlefield);
    let permanent = put(&mut g, Player::P0, "Forest", Zone::Library);
    let bad = put(&mut g, Player::P0, "Lightning Strike", Zone::Library);
    let other = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
    activate(&mut g, source, 0);
    reach(&mut g, |a| matches!(a, Action::SelectLibrary { .. }));
    assert!(g.observe(Player::P1).library_selection.is_none());
    reject(&mut g, Action::SelectLibrary { card: bad });
    act(&mut g, Action::SelectLibrary { card: permanent });
    act(&mut g, Action::FinishLibrarySelection);
    act(&mut g, Action::OrderLibraryBottom { card: bad });
    act(&mut g, Action::OrderLibraryBottom { card: other });
    drain(&mut g);
    assert_eq!(g.object_view(permanent).zone, Zone::Hand);
    assert!(
        g.observe(Player::P1)
            .known_hand
            .contains(&g.object_ref(permanent))
    );
    assert_eq!(&g.state.players[0].library[..2], &[other, bad]);
}
#[test]
fn lagorin_saddle_one_attack_puts_counters_on_up_to_two_mounts_or_vehicles_of_any_controller() {
    let mut g = scenario(Player::P0);
    let source = put(
        &mut g,
        Player::P0,
        "Lagorin, Soul of Alacria",
        Zone::Battlefield,
    );
    let rider = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let vehicle = put(
        &mut g,
        Player::P1,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let bad = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    activate(&mut g, source, 0);
    reject(&mut g, Action::FinishSaddle);
    let object = g.object_ref(rider);
    act(&mut g, Action::ChooseSaddler { object });
    act(&mut g, Action::FinishSaddle);
    drain(&mut g);
    assert!(g.object_view(rider).tapped);
    assert!(g.keywords(source).contains(Keywords::FLYING));
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: source });
    act(&mut g, Action::FinishAttackers);
    reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bad),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(vehicle),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    drain(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(vehicle, CounterKind::PlusOne), 1);
}
#[test]
fn lumbering_worldwagon_searches_on_entry_and_attack_and_its_power_tracks_lands_live() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Lumbering Worldwagon", Zone::Hand);
    cast(&mut g, source, None);
    reach(&mut g, |a| {
        matches!(a, Action::ChooseOptional { accept: true })
    });
    act(&mut g, Action::ChooseOptional { accept: true });
    reach(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
    let land = *g.state.players[0].library.last().unwrap();
    act(&mut g, Action::ChooseSearch { card: land });
    act(&mut g, Action::FinishSearch);
    drain(&mut g);
    assert!(g.object_view(land).tapped);
    let rider = put(&mut g, Player::P0, "Alacrian Jaguar", Zone::Battlefield);
    crew(&mut g, source, 0, &[rider]);
    assert_eq!(stats(&g, source), (1, 4));
    let second = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    assert_eq!(stats(&g, source), (2, 4));
    g.state.objects[source as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: source });
    act(&mut g, Action::FinishAttackers);
    reach(&mut g, |a| {
        matches!(a, Action::ChooseOptional { accept: false })
    });
    act(&mut g, Action::ChooseOptional { accept: false });
    drain(&mut g);
    assert_eq!(g.object_view(second).zone, Zone::Battlefield);
}
