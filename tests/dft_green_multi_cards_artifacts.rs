//! Artifact, entry, historical-count and resource effects from complete printed rules.
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
                    | Action::Discard { .. }
            )
        });
    }
    panic!("bounded artifact family resolution");
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
                    | Action::Discard { .. }
            )
        });
    }
    panic!("bounded artifact family decision");
}
fn begin(g: &mut Game, card: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn crew(g: &mut Game, card: CardId, power: u64) {
    let rider = put(g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    if power > 1 {
        g.add_counter(rider, CounterKind::PlusOne, power - 1);
    }
    begin(g, card, 0);
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
    let object = g.object_ref(rider);
    act(g, Action::ChooseCrewer { object });
    act(g, Action::FinishCrew);
    drain(g);
}
#[test]
fn remaining_vehicle_crew_costs_and_base_bodies_are_independently_wired() {
    for (name, power, body, keyword) in [
        ("Boosted Sloop", 1, (3, 3), Keywords::MENACE),
        ("Haunted Hellride", 1, (3, 3), Keywords::NONE),
        ("Cloudspire Skycycle", 1, (2, 3), Keywords::FLYING),
        ("Dune Drifter", 2, (3, 3), Keywords::NONE),
        ("Guidelight Pathmaker", 2, (6, 5), Keywords::VIGILANCE),
    ] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, name, Zone::Battlefield);
        crew(&mut g, source, power);
        assert_eq!(stats(&g, source), body, "{name}");
        assert!(g.keywords(source).contains(keyword), "{name}");
    }
}
#[test]
fn haunt_the_network_counts_its_new_flying_artifact_tokens_before_the_life_change() {
    let mut g = scenario(Player::P0);
    for _ in 0..2 {
        put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    }
    put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Haunt the Network", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(Player::P1)));
    drain(&mut g);
    assert_eq!(g.state.players[0].life, 24);
    assert_eq!(g.state.players[1].life, 16);
    let tokens: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(id as CardId, "Thopter"))
                .then_some(id as CardId)
        })
        .collect();
    assert_eq!(tokens.len(), 2);
    for id in tokens {
        assert_eq!(stats(&g, id), (1, 1));
        assert_eq!(g.object_view(id).colors, 0);
        assert!(g.keywords(id).contains(Keywords::FLYING));
        assert_ne!(g.object_view(id).kinds & 16, 0);
    }
}
#[test]
fn dredgers_insight_recovers_only_this_mill_batch_and_gains_life_only_for_artifact_or_creature_departure()
 {
    for (name, gain) in [("Grizzly Bears", 1), ("Swiftfoot Boots", 1), ("Forest", 0)] {
        let mut g = scenario(Player::P0);
        let old = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
        let bad = put(&mut g, Player::P0, "Lightning Strike", Zone::Library);
        put(&mut g, Player::P0, "Forest", Zone::Library);
        put(&mut g, Player::P0, "Forest", Zone::Library);
        let found = put(&mut g, Player::P0, name, Zone::Library);
        let length = g.state.players[0].library.len();
        let source = put(&mut g, Player::P0, "Dredger's Insight", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::SelectMilled { .. }));
        assert_eq!(g.state.players[0].library.len(), length - 4);
        for id in [old, bad] {
            let object = g.object_ref(id);
            reject(&mut g, Action::SelectMilled { object });
        }
        let object = g.object_ref(found);
        act(&mut g, Action::SelectMilled { object });
        drain(&mut g);
        assert_eq!(g.object_view(found).zone, Zone::Hand);
        assert_eq!(g.state.players[0].life, 20 + gain);
    }
}
#[test]
fn boosted_sloop_and_haunted_hellride_trigger_when_another_creature_attacks_without_crewing_them() {
    let mut g = scenario(Player::P0);
    let sloop = put(&mut g, Player::P0, "Boosted Sloop", Zone::Battlefield);
    let ride = put(&mut g, Player::P0, "Haunted Hellride", Zone::Battlefield);
    let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    put(&mut g, Player::P0, "Forest", Zone::Hand);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: bear });
    act(&mut g, Action::FinishAttackers);
    reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    drain(&mut g);
    assert!(!g.is_creature(sloop) && !g.is_creature(ride));
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.owner == Player::P0 && o.zone == Zone::Graveyard)
            .count(),
        1
    );
    assert_eq!(stats(&g, bear), (3, 2));
    assert!(!g.object_view(bear).tapped);
    assert!(g.keywords(bear).contains(Keywords::DEATHTOUCH));
    g.finish_cleanup();
    assert_eq!(stats(&g, bear), (2, 2));
    assert!(!g.keywords(bear).contains(Keywords::DEATHTOUCH));
}
#[test]
fn cloudspire_coordinator_scries_two_privately_and_uses_historical_entries_before_it_arrived() {
    let mut g = scenario(Player::P0);
    for name in ["Alacrian Jaguar", "Boosted Sloop"] {
        let card = put(&mut g, Player::P0, name, Zone::Hand);
        cast(&mut g, card, None);
        drain(&mut g);
    }
    let coordinator = put(&mut g, Player::P0, "Cloudspire Coordinator", Zone::Hand);
    cast(&mut g, coordinator, None);
    reach(&mut g, |a| matches!(a, Action::ArrangeLibrary { .. }));
    assert!(g.observe(Player::P1).library_choice.is_none());
    for _ in 0..2 {
        choose(&mut g, |a| {
            matches!(
                a,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Top,
                    ..
                }
            )
        });
    }
    drain(&mut g);
    assert_eq!(stats(&g, coordinator), (3, 1));
    // Model a separate haste-granting effect; the historical entry count stays this turn's count.
    g.state.objects[coordinator as usize].gained_keywords = Keywords::HASTE;
    begin(&mut g, coordinator, 0);
    pay(&mut g);
    drain(&mut g);
    let pilots: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(id as CardId, "Pilot"))
                .then_some(id as CardId)
        })
        .collect();
    assert_eq!(pilots.len(), 2);
    let vehicle = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, object)| object.card == Card::from_name("Boosted Sloop").unwrap())
        .unwrap()
        .0 as CardId;
    for id in pilots {
        assert_eq!(stats(&g, id), (1, 1));
        assert_eq!(g.crew_saddle_power(id, vehicle, "Vehicle"), 3);
    }
}
#[test]
fn cloudspire_skycycle_announces_two_counters_split_or_together_on_other_controlled_targets() {
    for split in [false, true] {
        let mut g = scenario(Player::P0);
        let vehicle = put(
            &mut g,
            Player::P0,
            "Cultivator's Caravan",
            Zone::Battlefield,
        );
        let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let source = put(&mut g, Player::P0, "Cloudspire Skycycle", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        for bad in [source, enemy] {
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
                target: Target::Card(vehicle),
            },
        );
        if split {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        act(
            &mut g,
            Action::AllocateTargetAmount {
                amount: if split { 1 } else { 2 },
            },
        );
        if split {
            act(&mut g, Action::AllocateTargetAmount { amount: 1 });
        }
        drain(&mut g);
        assert_eq!(
            g.counter_count(vehicle, CounterKind::PlusOne),
            if split { 1 } else { 2 }
        );
        assert_eq!(
            g.counter_count(bear, CounterKind::PlusOne),
            u64::from(split)
        );
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
    }
}
#[test]
fn broodheart_engine_surveils_at_its_upkeep_and_pays_sorcery_tap_sacrifice_to_return_creature_or_vehicle()
 {
    for vehicle in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Broodheart Engine", Zone::Battlefield);
        let target = put(
            &mut g,
            Player::P0,
            if vehicle {
                "Cultivator's Caravan"
            } else {
                "Grizzly Bears"
            },
            Zone::Graveyard,
        );
        let bad = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard);
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        reach(&mut g, |a| matches!(a, Action::ArrangeLibrary { .. }));
        let top = *g.state.players[0].library.last().unwrap();
        act(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Graveyard,
            },
        );
        drain(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Graveyard);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.step = Step::Main1;
        begin(&mut g, source, 0);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bad),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert!(!g.object_view(target).tapped);
    }
}
#[test]
fn dune_drifter_uses_cast_x_for_artifact_or_creature_recovery_but_noncast_return_has_x_zero() {
    let mut g = scenario(Player::P0);
    let boots = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard);
    let large = put(&mut g, Player::P0, "Hill Giant", Zone::Graveyard);
    let source = put(&mut g, Player::P0, "Dune Drifter", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: source });
    act(&mut g, Action::ChooseX { value: 2 });
    pay(&mut g);
    reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(large),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(boots),
        },
    );
    drain(&mut g);
    assert_eq!(g.object_view(boots).zone, Zone::Battlefield);
    g.move_object(source, Zone::Graveyard);
    let zero = put(&mut g, Player::P0, "Mox Jasper", Zone::Graveyard);
    let two = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let engine = put(&mut g, Player::P0, "Broodheart Engine", Zone::Battlefield);
    begin(&mut g, engine, 0);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    pay(&mut g);
    reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(two),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(zero),
        },
    );
    drain(&mut g);
    assert_eq!(g.object_view(zero).zone, Zone::Battlefield);
    assert_eq!(g.object_view(two).zone, Zone::Graveyard);
}
#[test]
fn guidelight_pathmaker_search_destination_uses_selected_artifact_value_and_decline_does_not_shuffle()
 {
    for name in [Some("Swiftfoot Boots"), Some("Cultivator's Caravan"), None] {
        let mut g = scenario(Player::P0);
        let found = put(
            &mut g,
            Player::P0,
            name.unwrap_or("Swiftfoot Boots"),
            Zone::Library,
        );
        let library = g.state.players[0].library.clone();
        let source = put(&mut g, Player::P0, "Guidelight Pathmaker", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::ChooseOptional { .. }));
        act(
            &mut g,
            Action::ChooseOptional {
                accept: name.is_some(),
            },
        );
        if let Some(name) = name {
            reach(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
            act(&mut g, Action::ChooseSearch { card: found });
            act(&mut g, Action::FinishSearch);
            drain(&mut g);
            assert_eq!(
                g.object_view(found).zone,
                if name == "Swiftfoot Boots" {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
        } else {
            drain(&mut g);
            assert_eq!(g.state.players[0].library, library);
        }
    }
}
#[test]
fn voyage_home_affinity_reduces_only_generic_cost_and_always_draws_three_gains_three() {
    for artifacts in [0u16, 2, 6] {
        let mut g = scenario(Player::P0);
        for _ in 0..artifacts {
            put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
        }
        put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Voyage Home", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(
            g.mana_payment_domain().unwrap().generic,
            5u16.saturating_sub(artifacts) as u32
        );
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
        assert_eq!(g.state.players[0].life, 23);
    }
}
#[test]
fn broadside_barrage_damages_creature_or_planeswalker_then_draws_and_discards_and_hazard_exhausts_once()
 {
    for planeswalker in [false, true] {
        let mut g = scenario(Player::P0);
        let target = put(
            &mut g,
            Player::P1,
            if planeswalker {
                "Ajani, Caller of the Pride"
            } else {
                "Hill Giant"
            },
            Zone::Battlefield,
        );
        if planeswalker {
            g.add_counter(target, CounterKind::Loyalty, 6);
        } else {
            g.add_counter(target, CounterKind::PlusOne, 3);
        }
        put(&mut g, Player::P0, "Forest", Zone::Hand);
        let spell = put(&mut g, Player::P0, "Broadside Barrage", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        drain(&mut g);
        if planeswalker {
            assert_eq!(g.counter_count(target, CounterKind::Loyalty), 1);
        } else {
            assert_eq!(g.object_view(target).damage, 5);
        }
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == Player::P0 && o.zone == Zone::Graveyard)
                .count(),
            2
        );
    }
    let mut g = scenario(Player::P0);
    let hazard = put(&mut g, Player::P0, "Hazard of the Dunes", Zone::Battlefield);
    assert!(
        g.keywords(hazard)
            .contains(Keywords::REACH.union(Keywords::TRAMPLE))
    );
    begin(&mut g, hazard, 0);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(stats(&g, hazard), (7, 7));
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: hazard,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
