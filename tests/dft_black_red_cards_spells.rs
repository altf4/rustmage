//! Complete printed obligations for the first black/red Aetherdrift spell family.
mod support;
use rustmage::Card;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn cleanup(g: &mut Game) {
    g.state.step = Step::End;
    g.advance_step();
}
fn mode_spell(g: &mut Game, name: &str, mode: u16, target: CardId) {
    let actor = g.state.active;
    let spell = put(g, actor, name, Zone::Hand);
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode });
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
fn crash_and_burn_has_distinct_vehicle_destruction_and_six_damage_modes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let vehicle = put(&mut g, p.other(), "Clamorous Ironclad", Zone::Battlefield);
        let artifact = put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let spell = put(&mut g, p, "Crash and Burn", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode: 0 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
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
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(vehicle).zone, Zone::Graveyard);
        assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
        for walker in [false, true] {
            let mut g = scenario(p);
            let target = put(
                &mut g,
                p.other(),
                if walker {
                    "Chandra, Spark Hunter"
                } else {
                    "Aegis Turtle"
                },
                Zone::Battlefield,
            );
            if walker {
                g.state.objects[target as usize].counters.loyalty = 10;
            } else {
                g.add_counter(target, CounterKind::PlusOne, 4);
            }
            mode_spell(&mut g, "Crash and Burn", 1, target);
            settle(&mut g);
            if walker {
                assert_eq!(g.object_view(target).counters.loyalty, 4);
            } else {
                assert_eq!(g.object_view(target).damage, 6);
            }
        }
    }
}

#[test]
fn skycrash_and_spin_out_destroy_only_their_printed_target_classes() {
    for (name, target_name, wrong_name) in [
        ("Skycrash", "Gilded Lotus", "Bear Cub"),
        ("Spin Out", "Clamorous Ironclad", "Gilded Lotus"),
        ("Spin Out", "Bear Cub", "Gilded Lotus"),
    ] {
        let mut g = scenario(Player::P0);
        let target = put(&mut g, Player::P1, target_name, Zone::Battlefield);
        let wrong = put(&mut g, Player::P1, wrong_name, Zone::Battlefield);
        let spell = put(&mut g, Player::P0, name, Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
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
        settle(&mut g);
        assert_eq!(
            g.object_view(target).zone,
            Zone::Graveyard,
            "{name}: {target_name}"
        );
        assert_eq!(g.object_view(wrong).zone, Zone::Battlefield);
    }
}

#[test]
fn printed_cycling_costs_discard_before_drawing_and_keep_owner_authority() {
    for p in [Player::P0, Player::P1] {
        for (name, mana) in [
            ("Fuel the Flames", [0, 0, 0, 0, 0, 2]),
            ("Locust Spray", [0, 0, 1, 0, 0, 0]),
            ("Skycrash", [0, 0, 0, 1, 0, 0]),
        ] {
            let mut g = scenario(p);
            let card = put(&mut g, p, name, Zone::Hand);
            let top = put(&mut g, p, "Shock", Zone::Library);
            let library = g.state.players[p.index()].library.len();
            g.state.players[p.index()].mana = [0; 6];
            let activate = Action::AnnounceActivate {
                card,
                ability: 0,
                counter_payment: Counters::default(),
            };
            assert!(
                !g.legal_actions().contains(&activate),
                "{name} requires its printed cost"
            );
            g.state.players[p.index()].mana = mana;
            act(&mut g, activate);
            pay(&mut g);
            assert_eq!(
                g.object_view(card).zone,
                Zone::Graveyard,
                "{name} discard is a cost"
            );
            assert_eq!(g.object_view(top).zone, Zone::Library);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(
                g.object_view(top).zone,
                Zone::Hand,
                "{name} draws exactly one"
            );
            assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        }
    }
}

#[test]
fn fuel_the_flames_damages_both_sides_creatures_but_not_uncrewed_vehicles_or_players() {
    let mut g = scenario(Player::P0);
    let own = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let other = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let survivor = put(&mut g, Player::P1, "Aegis Turtle", Zone::Battlefield);
    let vehicle = put(&mut g, Player::P1, "Clamorous Ironclad", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Fuel the Flames", Zone::Hand);
    cast(&mut g, spell, None);
    settle(&mut g);
    assert_eq!(g.object_view(own).zone, Zone::Graveyard);
    assert_eq!(g.object_view(other).zone, Zone::Graveyard);
    assert_eq!(g.object_view(survivor).damage, 2);
    assert_eq!(g.object_view(vehicle).zone, Zone::Battlefield);
    assert_eq!(g.object_view(vehicle).damage, 0);
    assert_eq!([g.state.players[0].life, g.state.players[1].life], [20, 20]);
}

#[test]
fn locust_spray_reduces_both_stats_until_cleanup_and_can_kill_one_toughness() {
    for (target_name, survives) in [("Bear Cub", true), ("Healer's Hawk", false)] {
        let mut g = scenario(Player::P0);
        let target = put(&mut g, Player::P1, target_name, Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Locust Spray", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        settle(&mut g);
        if survives {
            assert_eq!(stats(&g, target), (1, 1));
            cleanup(&mut g);
            assert_eq!(stats(&g, target), (2, 2));
        } else {
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn maximum_overdrive_keeps_its_counter_after_temporary_keywords_expire() {
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Maximum Overdrive", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    settle(&mut g);
    assert_eq!(stats(&g, target), (3, 3));
    assert!(g.keywords(target).contains(Keywords::DEATHTOUCH));
    assert!(g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
    let damage = put(&mut g, Player::P0, "Lightning Strike", Zone::Hand);
    cast(&mut g, damage, Some(Target::Card(target)));
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    assert_eq!(g.object_view(target).damage, 3);
    cleanup(&mut g);
    assert_eq!(stats(&g, target), (3, 3));
    assert!(!g.keywords(target).contains(Keywords::DEATHTOUCH));
    assert!(!g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn pedal_to_the_metal_locks_paid_x_and_grants_first_strike_even_at_zero() {
    for x in [0, 3] {
        let mut g = scenario(Player::P0);
        let target = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Pedal to the Metal", Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 1, 0, x];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseX { value: x });
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
        settle(&mut g);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(stats(&g, target), (2 + i128::from(x), 2));
        assert!(g.keywords(target).contains(Keywords::FIRST_STRIKE));
        cleanup(&mut g);
        assert_eq!(stats(&g, target), (2, 2));
        assert!(!g.keywords(target).contains(Keywords::FIRST_STRIKE));
    }
}

#[test]
fn quag_feast_uses_post_mill_graveyard_count_and_accepts_all_three_target_types() {
    for (name, grave_before, destroyed) in [
        ("Bear Cub", 0, true),
        ("Clamorous Ironclad", 2, true),
        ("Chandra, Spark Hunter", 1, false),
        ("Chandra, Spark Hunter", 2, true),
    ] {
        let mut g = scenario(Player::P0);
        let target = put(&mut g, Player::P1, name, Zone::Battlefield);
        if name == "Chandra, Spark Hunter" {
            g.state.objects[target as usize].counters.loyalty = 8;
        }
        for _ in 0..grave_before {
            put(&mut g, Player::P0, "Forest", Zone::Graveyard);
        }
        let library = g.state.players[0].library.len();
        let spell = put(&mut g, Player::P0, "Quag Feast", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        settle(&mut g);
        assert_eq!(g.state.players[0].library.len(), library - 2);
        assert_eq!(
            g.object_view(target).zone == Zone::Graveyard,
            destroyed,
            "{name}, {grave_before}"
        );
    }
}

#[test]
fn quag_feast_does_not_mill_when_its_only_target_is_a_new_incarnation() {
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Quag Feast", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    let library = g.state.players[0].library.len();
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.state.players[0].library.len(), library);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
}

#[test]
fn road_rage_counts_current_controlled_union_once_at_resolution() {
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Clamorous Ironclad", Zone::Battlefield);
    g.change_object_form(
        g.object_ref(vehicle),
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    let item = g.cast_item(Player::P0, vehicle, [0; 6]);
    g.add_creature_types(&item, Subject::Source, &["Mount"]);
    let mount = put(&mut g, Player::P0, "Trained Arynx", Zone::Battlefield);
    put(&mut g, Player::P1, "Trained Arynx", Zone::Battlefield);
    put(&mut g, Player::P1, "Clamorous Ironclad", Zone::Battlefield);
    let target = put(
        &mut g,
        Player::P1,
        "Chandra, Spark Hunter",
        Zone::Battlefield,
    );
    g.state.objects[target as usize].counters.loyalty = 10;
    let spell = put(&mut g, Player::P0, "Road Rage", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    g.move_object(mount, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(
        g.object_view(target).counters.loyalty,
        7,
        "2 + one dual-subtype permanent"
    );
}

#[test]
fn risky_shortcut_draws_two_then_both_players_lose_two_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let top1 = put(&mut g, p, "Shock", Zone::Library);
        let top2 = put(&mut g, p, "Divination", Zone::Library);
        let library = g.state.players[p.index()].library.len();
        let spell = put(&mut g, p, "Risky Shortcut", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.object_view(top1).zone, Zone::Hand);
        assert_eq!(g.object_view(top2).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].library.len(), library - 2);
        assert_eq!([g.state.players[0].life, g.state.players[1].life], [18, 18]);
    }
}

#[test]
fn syphon_fuel_shrinks_six_and_gains_two_only_when_target_remains_legal() {
    for stale in [false, true] {
        let mut g = scenario(Player::P0);
        let target = put(&mut g, Player::P1, "Aegis Turtle", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Syphon Fuel", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        if stale {
            g.move_object(target, Zone::Hand);
            g.move_object(target, Zone::Battlefield);
        }
        settle(&mut g);
        assert_eq!(g.state.players[0].life, if stale { 20 } else { 22 });
        assert_eq!(
            g.object_view(target).zone,
            if stale {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn syphon_fuel_subtracts_exactly_six_from_a_surviving_creatures_two_stats() {
    let mut g = scenario(Player::P0);
    let target = put(&mut g, Player::P1, "Aegis Turtle", Zone::Battlefield);
    g.add_counter(target, CounterKind::PlusOne, 8);
    let spell = put(&mut g, Player::P0, "Syphon Fuel", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(target)));
    settle(&mut g);
    assert_eq!(stats(&g, target), (2, 7));
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn lightning_strike_deals_three_to_a_player_and_planeswalker() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        let walker = put(
            &mut g,
            p.other(),
            "Chandra, Spark Hunter",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 8;
        let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(walker)));
        settle(&mut g);
        assert_eq!(g.object_view(walker).counters.loyalty, 5);
    }
}
