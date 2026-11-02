//! Actual green search, recovery, attachment and modal spell outcomes.
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
    panic!("bounded green spell resolution");
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
    panic!("bounded green resolution choice");
}
fn spell(g: &mut Game, name: &str, mode: Option<u16>, targets: &[CardId]) -> CardId {
    let card = put(g, Player::P0, name, Zone::Hand);
    act(g, Action::AnnounceCast { card });
    if let Some(mode) = mode {
        act(g, Action::ChooseMode { mode });
    }
    for &id in targets {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    for _ in 0..5 {
        if !g.legal_actions().contains(&Action::FinishTargets) {
            break;
        }
        act(g, Action::FinishTargets);
    }
    pay(g);
    drain(g);
    card
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
    pay(g);
}
#[test]
fn bestow_greatness_grants_exact_stats_and_trample_until_cleanup() {
    let mut g = scenario(Player::P0);
    let bear = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    spell(&mut g, "Bestow Greatness", None, &[bear]);
    assert_eq!(stats(&g, bear), (6, 6));
    assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
    g.finish_cleanup();
    assert_eq!(stats(&g, bear), (2, 2));
    assert!(!g.keywords(bear).contains(Keywords::TRAMPLE));
}
#[test]
fn broken_wings_accepts_each_printed_kind_and_rejects_nonflying_creature() {
    for name in ["Swiftfoot Boots", "Doubling Season", "Serra Angel"] {
        let mut g = scenario(Player::P0);
        let target = put(&mut g, Player::P1, name, Zone::Battlefield);
        let bear = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let card = put(&mut g, Player::P0, "Broken Wings", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    }
}
#[test]
fn defend_the_rider_protects_a_land_or_creates_a_pilot_that_really_crews_for_three() {
    let mut g = scenario(Player::P0);
    let land = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Forest", Zone::Battlefield);
    let card = put(&mut g, Player::P0, "Defend the Rider", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card });
    act(&mut g, Action::ChooseMode { mode: 0 });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    pay(&mut g);
    drain(&mut g);
    assert!(
        g.keywords(land)
            .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
    );
    g.finish_cleanup();
    assert!(
        !g.keywords(land)
            .intersects(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
    );
    let mut g = scenario(Player::P0);
    let vehicle = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    spell(&mut g, "Defend the Rider", Some(1), &[]);
    let pilot = g
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(id as CardId, "Pilot"))
                .then_some(id as CardId)
        })
        .unwrap();
    assert_eq!(stats(&g, pilot), (1, 1));
    assert_eq!(g.object_view(pilot).colors, 0);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: vehicle,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    let object = g.object_ref(pilot);
    act(&mut g, Action::ChooseCrewer { object });
    act(&mut g, Action::FinishCrew);
    drain(&mut g);
    assert!(g.is_creature(vehicle));
    assert!(g.object_view(pilot).tapped);
}
#[test]
fn plow_through_has_independent_fight_and_uncrewed_vehicle_destruction_modes() {
    let mut g = scenario(Player::P0);
    let own = put(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    spell(&mut g, "Plow Through", Some(0), &[own, enemy]);
    assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(g.object_view(own).damage, 2);
    let vehicle = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    assert!(!g.is_creature(vehicle));
    spell(&mut g, "Plow Through", Some(1), &[vehicle]);
    assert_eq!(g.object_view(vehicle).zone, Zone::Graveyard);
}
#[test]
fn run_over_reduces_mount_cost_and_deals_one_way_damage_while_plain_creature_pays_full() {
    for mount in [true, false] {
        let mut g = scenario(Player::P0);
        let own = put(
            &mut g,
            Player::P0,
            if mount {
                "Alacrian Jaguar"
            } else {
                "Grizzly Bears"
            },
            Zone::Battlefield,
        );
        let enemy = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        g.add_counter(enemy, CounterKind::PlusOne, 5);
        let card = put(&mut g, Player::P0, "Run Over", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        assert_eq!(
            g.mana_payment_domain().unwrap().generic,
            if mount { 0 } else { 1 }
        );
        let power = stats(&g, own).0;
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(own).damage, 0);
        assert_eq!(g.object_view(enemy).damage, power);
    }
}
#[test]
fn silken_strength_flashes_onto_creature_or_vehicle_untaps_and_grants_live_bonuses() {
    for vehicle in [false, true] {
        let mut g = scenario(Player::P0);
        let target = put(
            &mut g,
            Player::P0,
            if vehicle {
                "Cultivator's Caravan"
            } else {
                "Grizzly Bears"
            },
            Zone::Battlefield,
        );
        let bad = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
        g.state.objects[target as usize].tapped = true;
        g.state.active = Player::P1;
        g.state.step = Step::Upkeep;
        g.state.priority = Player::P0;
        let aura = put(&mut g, Player::P0, "Silken Strength", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: aura });
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
        drain(&mut g);
        assert!(!g.object_view(target).tapped);
        assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(target)));
        assert!(g.keywords(target).contains(Keywords::REACH));
        if vehicle {
            assert!(!g.is_creature(target));
            g.change_object_form(
                g.object_ref(target),
                AnimationSpec {
                    artifact: true,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: None,
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
            assert_eq!(stats(&g, target), (6, 7));
        } else {
            assert_eq!(stats(&g, target), (3, 4));
        }
    }
}
#[test]
fn molt_tender_mills_one_and_pays_its_tap_and_graveyard_exile_for_colored_mana() {
    let mut g = scenario(Player::P0);
    let tender = put(&mut g, Player::P0, "Molt Tender", Zone::Battlefield);
    let top = *g.state.players[0].library.last().unwrap();
    activate(&mut g, tender, 0);
    drain(&mut g);
    assert_eq!(g.object_view(top).zone, Zone::Graveyard);
    assert!(g.object_view(tender).tapped);
    g.state.objects[tender as usize].tapped = false;
    g.state.players[0].mana = [0; 6];
    let foreign = put(&mut g, Player::P1, "Forest", Zone::Graveyard);
    let output = [0, 1, 0, 0, 0, 0];
    act(
        &mut g,
        Action::AnnounceMana {
            card: tender,
            ability: 1,
            counter_payment: Counters::default(),
            production: output,
        },
    );
    let object = g.object_ref(foreign);
    reject(&mut g, Action::ChooseAdditionalCost { object });
    let object = g.object_ref(top);
    act(&mut g, Action::ChooseAdditionalCost { object });
    if g.legal_actions()
        .contains(&Action::FinishAdditionalCostGroup)
    {
        act(&mut g, Action::FinishAdditionalCostGroup);
    }
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    drain(&mut g);
    assert_eq!(g.object_view(top).zone, Zone::Exile);
    assert!(g.object_view(tender).tapped);
    assert_eq!(g.state.players[0].mana, output);
}
#[test]
fn fang_druid_searches_either_zone_for_only_one_abilityless_creature_and_reveals_result() {
    for library in [true, false] {
        let mut g = scenario(Player::P0);
        let found = put(
            &mut g,
            Player::P0,
            "Jibbirik Omnivore",
            if library {
                Zone::Library
            } else {
                Zone::Graveyard
            },
        );
        let cycling = put(&mut g, Player::P0, "Migrating Ketradon", Zone::Graveyard);
        let card = put(&mut g, Player::P0, "Fang-Druid Summoner", Zone::Hand);
        cast(&mut g, card, None);
        reach(&mut g, |a| matches!(a, Action::ChooseSearchZones { .. }));
        assert_eq!(stats(&g, card), (2, 4));
        assert!(g.keywords(card).contains(Keywords::REACH));
        act(
            &mut g,
            Action::ChooseSearchZones {
                library,
                graveyard: true,
            },
        );
        reject(&mut g, Action::ChooseSearch { card: cycling });
        assert!(g.observe(Player::P1).zone_search.unwrap().cards.is_none());
        act(&mut g, Action::ChooseSearch { card: found });
        act(&mut g, Action::FinishSearch);
        drain(&mut g);
        assert_eq!(g.object_view(found).zone, Zone::Hand);
        assert!(
            g.observe(Player::P1)
                .known_hand
                .contains(&g.object_ref(found))
        );
    }
}
#[test]
fn fang_druid_can_decline_without_shuffling_or_recovering_an_eligible_card() {
    let mut g = scenario(Player::P0);
    let grave = put(&mut g, Player::P0, "Terrian, World Tyrant", Zone::Graveyard);
    let library = g.state.players[0].library.clone();
    let source = put(&mut g, Player::P0, "Fang-Druid Summoner", Zone::Hand);
    cast(&mut g, source, None);
    reach(&mut g, |a| matches!(a, Action::ChooseSearchZones { .. }));
    act(
        &mut g,
        Action::ChooseSearchZones {
            library: false,
            graveyard: false,
        },
    );
    act(&mut g, Action::FinishSearch);
    drain(&mut g);
    assert_eq!(g.state.players[0].library, library);
    assert_eq!(g.object_view(grave).zone, Zone::Graveyard);
}
#[test]
fn pothole_mole_mills_three_then_can_recover_an_old_land_or_decline() {
    for recover in [false, true] {
        let mut g = scenario(Player::P0);
        let old = put(&mut g, Player::P0, "Mountain", Zone::Graveyard);
        let bad = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
        let before = g.state.players[0].library.len();
        let source = put(&mut g, Player::P0, "Pothole Mole", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::FinishMilledSelection));
        assert_eq!(g.state.players[0].library.len(), before - 3);
        let object = g.object_ref(bad);
        reject(&mut g, Action::SelectMilled { object });
        if recover {
            let object = g.object_ref(old);
            act(&mut g, Action::SelectMilled { object });
        }
        if g.legal_actions().contains(&Action::FinishMilledSelection) {
            act(&mut g, Action::FinishMilledSelection);
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(old).zone,
            if recover { Zone::Hand } else { Zone::Graveyard }
        );
        assert_eq!(stats(&g, source), (2, 3));
    }
}
#[test]
fn point_the_way_starts_speed_and_sacrifices_for_up_to_current_speed_tapped_basics() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Point the Way", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].speed, 1);
    g.state.players[0].speed = 3;
    let lands = g.state.players[0].library.clone();
    activate(&mut g, source, 0);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    reach(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
    for id in &lands[..3] {
        act(&mut g, Action::ChooseSearch { card: *id });
    }
    reject(&mut g, Action::ChooseSearch { card: lands[3] });
    act(&mut g, Action::FinishSearch);
    drain(&mut g);
    for id in &lands[..3] {
        assert_eq!(g.object_view(*id).zone, Zone::Battlefield);
        assert!(g.object_view(*id).tapped);
    }
}
#[test]
fn rise_from_the_wreck_selects_all_four_printed_groups_and_returns_each_card() {
    let mut g = scenario(Player::P0);
    let cards = [
        "Migrating Ketradon",
        "Alacrian Jaguar",
        "Cultivator's Caravan",
        "Jibbirik Omnivore",
    ]
    .map(|n| put(&mut g, Player::P0, n, Zone::Graveyard));
    spell(&mut g, "Rise from the Wreck", None, &cards);
    for id in cards {
        assert_eq!(g.object_view(id).zone, Zone::Hand);
    }
}
#[test]
fn skyserpent_reveals_through_two_lands_puts_them_tapped_and_adds_a_counter_even_if_empty() {
    for count in [0, 2] {
        let mut g = scenario(Player::P0);
        for object in &mut g.state.objects {
            if object.zone == Zone::Library && object.owner == Player::P0 {
                object.zone = Zone::Exile;
            }
        }
        g.state.players[0].library.clear();
        let mut lands = Vec::new();
        let mut rest = Vec::new();
        for _ in 0..count {
            rest.push(put(&mut g, Player::P0, "Grizzly Bears", Zone::Library));
            lands.push(put(&mut g, Player::P0, "Forest", Zone::Library));
        }
        let source = put(&mut g, Player::P0, "Skyserpent Seeker", Zone::Battlefield);
        assert!(
            g.keywords(source)
                .contains(Keywords::FLYING.union(Keywords::DEATHTOUCH))
        );
        activate(&mut g, source, 0);
        if count > 0 {
            reach(&mut g, |a| matches!(a, Action::SelectLibrary { .. }));
            for id in &lands {
                act(&mut g, Action::SelectLibrary { card: *id });
            }
            act(&mut g, Action::FinishLibrarySelection);
        }
        drain(&mut g);
        for id in lands {
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert!(g.object_view(id).tapped);
        }
        for id in rest {
            assert_eq!(g.object_view(id).zone, Zone::Library);
        }
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
