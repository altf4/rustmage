//! Real W/U permanent outcomes: entry effects, live characteristics and paid costs.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
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
    panic!("permanent family exceeded 64 actions");
}
fn target_ready(g: &mut Game, id: CardId) {
    for _ in 0..32 {
        if g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(id),
        }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("expected target choice");
}
fn finish_targets(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn crew(g: &mut Game, vehicle: CardId, contributors: &[CardId]) {
    act(
        g,
        Action::AnnounceActivate {
            card: vehicle,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
    for &id in contributors {
        let object = g.object_ref(id);
        act(g, Action::ChooseCrewer { object });
    }
    act(g, Action::FinishCrew);
    drain(g);
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let id = i as CardId;
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id, subtype))
            .then_some(id)
        })
        .collect()
}

#[test]
fn broadcast_and_nimble_entries_create_one_real_colorless_flying_artifact_thopter() {
    for p in [Player::P0, Player::P1] {
        for name in ["Broadcast Rambler", "Nimble Thopterist"] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None);
            drain(&mut g);
            let made = tokens(&g, p, "Thopter");
            assert_eq!(made.len(), 1, "{name}");
            let token = made[0];
            assert_eq!(stats(&g, token), (1, 1));
            assert_eq!(g.object_view(token).colors, 0);
            assert_eq!(g.current_kinds(token), 1 | 16);
            assert!(g.keywords(token).contains(Keywords::FLYING));
            if name == "Nimble Thopterist" {
                assert_eq!(stats(&g, source), (3, 2));
            } else {
                assert!(!g.is_creature(source));
                crew(&mut g, source, &[token]);
                assert_eq!(stats(&g, source), (5, 4));
                assert!(g.object_view(token).tapped);
            }
        }
    }
}

#[test]
fn air_response_hulldrifter_and_midnight_mangler_have_their_real_crew_bodies() {
    for (name, minimum, body, keywords) in [
        (
            "Air Response Unit",
            1,
            (3, 3),
            Keywords::FLYING.union(Keywords::VIGILANCE),
        ),
        ("Hulldrifter", 3, (3, 2), Keywords::FLYING),
        ("Midnight Mangler", 2, (3, 3), Keywords::NONE),
    ] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, name, Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert!(!g.is_creature(source), "{name}");
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            if name == "Hulldrifter" { 2 } else { 0 }
        );
        let pilot = put(
            &mut g,
            Player::P0,
            if minimum == 3 {
                "Hill Giant"
            } else {
                "Grizzly Bears"
            },
            Zone::Battlefield,
        );
        crew(&mut g, source, &[pilot]);
        assert_eq!(stats(&g, source), body, "{name}");
        assert!(g.keywords(source).contains(keywords));
        g.finish_cleanup();
        g.state.active = Player::P0;
        assert!(!g.is_creature(source));
        if name == "Midnight Mangler" {
            g.state.active = Player::P1;
            assert!(g.is_creature(source));
            assert_eq!(stats(&g, source), (3, 3));
            g.state.objects[source as usize].controller = Player::P1;
            assert!(!g.is_creature(source));
        }
    }
}

#[test]
fn spotcycle_scouter_scries_exactly_two_privately_then_crews_as_a_three_two() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bottom = put(&mut g, p, "Shock", Zone::Library);
    let top = put(&mut g, p, "Grizzly Bears", Zone::Library);
    let source = put(&mut g, p, "Spotcycle Scouter", Zone::Hand);
    cast(&mut g, source, None);
    for _ in 0..32 {
        if g.state.pending != Pending::Priority {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert_eq!(
        g.observe(p)
            .library_choice
            .as_ref()
            .unwrap()
            .remaining
            .len(),
        2
    );
    assert!(g.observe(p.other()).library_choice.is_none());
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: bottom,
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
    drain(&mut g);
    assert_eq!(g.top_library_card(p), Some(top));
    assert_eq!(g.state.players[0].library.first(), Some(&bottom));
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    let contributor = put(&mut g, p, "Savannah Lions", Zone::Battlefield);
    crew(&mut g, source, &[contributor]);
    assert_eq!(stats(&g, source), (3, 2));
}

#[test]
fn artifact_affinity_pays_only_the_remaining_generic_and_required_blue() {
    for (name, mana, body, flying) in [
        ("Memory Guardian", [0, 1, 0, 0, 0, 1], (3, 4), true),
        ("Gearseeker Serpent", [0, 2, 0, 0, 0, 2], (5, 6), false),
    ] {
        let mut g = scenario(Player::P0);
        for _ in 0..3 {
            put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
        }
        for _ in 0..4 {
            put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
        }
        let source = put(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = mana;
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(stats(&g, source), body);
        assert_eq!(g.keywords(source).contains(Keywords::FLYING), flying);
        if name == "Gearseeker Serpent" {
            g.state.players[0].mana = [0, 1, 0, 0, 0, 5];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            drain(&mut g);
            assert_eq!(g.state.players[0].mana, [0; 6]);
            assert!(g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
            g.finish_cleanup();
            assert!(!g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
        }
    }
}

#[test]
fn guidelight_synergist_tracks_only_its_live_controllers_artifact_count() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Guidelight Synergist", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(stats(&g, source), (1, 4));
    assert!(g.keywords(source).contains(Keywords::FLYING));
    let own = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    assert_eq!(stats(&g, source), (2, 4));
    g.move_object(own, Zone::Graveyard);
    assert_eq!(stats(&g, source), (1, 4));
    g.state.objects[source as usize].controller = Player::P1;
    assert_eq!(stats(&g, source), (2, 4));
}

#[test]
fn quickwelder_reduces_only_artifact_spells_of_its_controller() {
    let p = Player::P0;
    let mut g = scenario(p);
    let welder = put(&mut g, p, "Voyager Quickwelder", Zone::Hand);
    cast(&mut g, welder, None);
    drain(&mut g);
    assert_eq!(stats(&g, welder), (2, 4));
    let artifact = put(&mut g, p, "Swiftfoot Boots", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    cast(&mut g, artifact, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
    let spell = put(&mut g, p, "Divination", Zone::Hand);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 1];
    reject(&mut g, Action::AnnounceCast { card: spell });
    g.state.objects[welder as usize].controller = Player::P1;
    let other = put(&mut g, p, "Swiftfoot Boots", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    reject(&mut g, Action::AnnounceCast { card: other });
}

#[test]
fn daring_mechanic_can_counter_opposing_mounts_and_uncrewed_vehicles_but_not_other_creatures() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Daring Mechanic", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(stats(&g, source), (3, 3));
    let wrong = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let vehicle = put(&mut g, p.other(), "Clamorous Ironclad", Zone::Battlefield);
    let mount = put(&mut g, p.other(), "Trained Arynx", Zone::Battlefield);
    for id in [vehicle, mount] {
        g.state.players[0].mana = [1, 0, 0, 0, 0, 3];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
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
                target: Target::Card(id),
            },
        );
        finish_targets(&mut g);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert!(!g.object_view(source).tapped);
    }
    assert!(!g.is_creature(vehicle));
    assert_eq!(stats(&g, mount), (4, 2));
}

#[test]
fn lotusguard_entry_grants_both_keywords_to_an_uncrewed_vehicle_independent_of_source() {
    let p = Player::P0;
    let mut g = scenario(p);
    let vehicle = put(&mut g, p.other(), "Clamorous Ironclad", Zone::Battlefield);
    let source = put(&mut g, p, "Lotusguard Disciple", Zone::Hand);
    cast(&mut g, source, None);
    target_ready(&mut g, vehicle);
    assert_eq!(stats(&g, source), (2, 2));
    assert!(g.keywords(source).contains(Keywords::FLYING));
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(vehicle),
        },
    );
    finish_targets(&mut g);
    g.move_object(source, Zone::Graveyard);
    drain(&mut g);
    assert!(!g.is_creature(vehicle));
    assert!(
        g.keywords(vehicle)
            .contains(Keywords::LIFELINK.union(Keywords::INDESTRUCTIBLE))
    );
    g.finish_cleanup();
    assert!(!g.keywords(vehicle).contains(Keywords::LIFELINK));
    assert!(!g.keywords(vehicle).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn roadside_assistance_grants_stats_lifelink_and_a_colorless_pilot_that_can_crew_three() {
    let p = Player::P0;
    let mut g = scenario(p);
    let vehicle = put(&mut g, p, "Hulldrifter", Zone::Battlefield);
    let source = put(&mut g, p, "Roadside Assistance", Zone::Hand);
    cast(&mut g, source, Some(Target::Card(vehicle)));
    drain(&mut g);
    let made = tokens(&g, p, "Pilot");
    assert_eq!(made.len(), 1);
    let pilot = made[0];
    assert_eq!(stats(&g, pilot), (1, 1));
    assert_eq!(g.object_view(pilot).colors, 0);
    assert_eq!(g.current_kinds(pilot), 1);
    assert!(g.keywords(pilot).contains(Keywords::CREW_SADDLE_PLUS_TWO));
    assert!(!g.is_creature(vehicle));
    assert!(g.keywords(vehicle).contains(Keywords::LIFELINK));
    crew(&mut g, vehicle, &[pilot]);
    assert_eq!(stats(&g, vehicle), (4, 3));
}

#[test]
fn lightwheel_starts_speed_enchants_a_vehicle_and_casts_normally_from_own_graveyard_at_four() {
    let p = Player::P0;
    let mut g = scenario(p);
    let vehicle = put(&mut g, p, "Air Response Unit", Zone::Battlefield);
    let source = put(&mut g, p, "Lightwheel Enhancements", Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 0];
    cast(&mut g, source, Some(Target::Card(vehicle)));
    drain(&mut g);
    assert_eq!(g.state.players[0].speed, 1);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert!(g.keywords(vehicle).contains(Keywords::VIGILANCE));
    let pilot = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    crew(&mut g, vehicle, &[pilot]);
    assert_eq!(stats(&g, vehicle), (4, 4));
    g.move_object(source, Zone::Graveyard);
    g.state.players[0].speed = 3;
    g.state.players[0].mana = [1, 0, 0, 0, 0, 0];
    reject(&mut g, Action::AnnounceCast { card: source });
    g.state.players[0].speed = 4;
    cast(&mut g, source, Some(Target::Card(pilot)));
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Normal
    );
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    assert_eq!(stats(&g, pilot), (3, 3));
    assert!(g.keywords(pilot).contains(Keywords::VIGILANCE));
    assert!(!g.state.objects[source as usize].flashback);
}

#[test]
fn unstoppable_plan_untaps_own_nonlands_only_at_its_controllers_end_step() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Unstoppable Plan", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    let creature = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
    let land = put(&mut g, p, "Forest", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    for id in [creature, artifact, land, other] {
        g.state.objects[id as usize].tapped = true;
    }
    g.begin_step(Step::End);
    g.give_priority();
    drain(&mut g);
    assert!(!g.object_view(creature).tapped && !g.object_view(artifact).tapped);
    assert!(g.object_view(land).tapped && g.object_view(other).tapped);
    g.state.objects[creature as usize].tapped = true;
    g.state.active = p.other();
    g.begin_step(Step::End);
    g.give_priority();
    drain(&mut g);
    assert!(g.object_view(creature).tapped);
}
