//! Paid outcomes for white/blue removal, selection, cycling and exchange spells.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("tempo resolution exceeded 64 actions");
}
fn reach_choice(g: &mut Game) {
    for _ in 0..32 {
        if g.state.pending != Pending::Priority {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("expected bounded spell choice");
}
fn finish_targets(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn modal(g: &mut Game, source: CardId, mode: u16, target: CardId) {
    act(g, Action::AnnounceCast { card: source });
    act(g, Action::ChooseMode { mode });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish_targets(g);
    pay(g);
}
fn animate(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    g.change_object_form(
        object,
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
}

#[test]
fn collision_course_counts_the_creature_vehicle_union_once_and_has_an_artifact_mode() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let both = put(&mut g, p, "Veloheart Bike", Zone::Battlefield);
        animate(&mut g, both);
        let target = put(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let artifact = put(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
        let source = put(&mut g, p, "Collision Course", Zone::Hand);
        modal(&mut g, source, 0, target);
        drain(&mut g);
        assert_eq!(g.object_view(target).damage, 3);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        let source = put(&mut g, p, "Collision Course", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        act(&mut g, Action::ChooseMode { mode: 1 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        finish_targets(&mut g);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
    }
}

#[test]
fn flood_the_engine_taps_blanks_and_locks_an_uncrewed_vehicle_until_the_aura_leaves() {
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P1, "Veloheart Bike", Zone::Battlefield);
    let wrong = put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    let source = put(&mut g, Player::P0, "Flood the Engine", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: source });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(wrong),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(vehicle),
        },
    );
    finish_targets(&mut g);
    pay(&mut g);
    drain(&mut g);
    assert!(g.object_view(vehicle).tapped);
    assert!(!g.is_creature(vehicle));
    assert!(!g.printed_abilities_available(vehicle));
    g.state.active = Player::P1;
    g.untap_active_player();
    assert!(g.object_view(vehicle).tapped);
    g.move_object(source, Zone::Graveyard);
    assert!(g.printed_abilities_available(vehicle));
    g.untap_active_player();
    assert!(!g.object_view(vehicle).tapped);
}

#[test]
fn stall_out_puts_three_real_stun_counters_on_a_vehicle() {
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P1, "Clamorous Ironclad", Zone::Battlefield);
    let source = put(&mut g, Player::P0, "Stall Out", Zone::Hand);
    cast(&mut g, source, Some(Target::Card(vehicle)));
    drain(&mut g);
    assert!(g.object_view(vehicle).tapped);
    assert_eq!(g.counter_count(vehicle, CounterKind::Stun), 3);
    g.state.active = Player::P1;
    for remaining in [2, 1, 0] {
        g.untap_active_player();
        assert!(g.object_view(vehicle).tapped);
        assert_eq!(g.counter_count(vehicle, CounterKind::Stun), remaining);
    }
    g.untap_active_player();
    assert!(!g.object_view(vehicle).tapped);
}

#[test]
fn spectacular_pileup_removes_indestructible_only_from_the_destroyed_union() {
    let mut g = scenario(Player::P0);
    let own = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut g, Player::P1, "Aegis Turtle", Zone::Battlefield);
    let vehicle = put(&mut g, Player::P1, "Clamorous Ironclad", Zone::Battlefield);
    let survivor = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    for id in [own, other, vehicle, survivor] {
        g.state.objects[id as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    }
    let source = put(&mut g, Player::P0, "Spectacular Pileup", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    for id in [own, other, vehicle] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(g.object_view(survivor).zone, Zone::Battlefield);
    assert!(g.keywords(survivor).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn trade_the_helm_exchanges_both_objects_or_neither_when_one_incarnation_is_lost() {
    for stale in [false, true] {
        let mut g = scenario(Player::P0);
        let own = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
        let other = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let source = put(&mut g, Player::P0, "Trade the Helm", Zone::Hand);
        announce(&mut g, source, &[Target::Card(own), Target::Card(other)]);
        if stale {
            g.move_object(other, Zone::Exile);
            g.move_object(other, Zone::Battlefield);
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(own).controller,
            if stale { Player::P0 } else { Player::P1 }
        );
        assert_eq!(
            g.object_view(other).controller,
            if stale { Player::P1 } else { Player::P0 }
        );
        g.finish_cleanup();
        assert_eq!(
            g.object_view(own).controller,
            if stale { Player::P0 } else { Player::P1 }
        );
    }
}

#[test]
fn tune_up_returns_the_owned_artifact_and_animates_only_its_exact_vehicle_entry() {
    for name in ["Veloheart Bike", "Swiftfoot Boots"] {
        let mut g = scenario(Player::P0);
        let own = put(&mut g, Player::P0, name, Zone::Graveyard);
        let old = g.object_ref(own);
        let other = put(&mut g, Player::P1, "Veloheart Bike", Zone::Graveyard);
        let source = put(&mut g, Player::P0, "Tune Up", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        finish_targets(&mut g);
        pay(&mut g);
        drain(&mut g);
        assert_ne!(g.object_ref(own), old);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        assert_eq!(g.is_creature(own), name == "Veloheart Bike");
        if name == "Veloheart Bike" {
            assert_eq!(stats(&g, own), (4, 2));
        }
        g.finish_cleanup();
        assert_eq!(g.is_creature(own), name == "Veloheart Bike");
    }
}

#[test]
fn tempo_family_cycling_pays_two_and_discards_before_exactly_one_draw() {
    for name in [
        "Gallant Strike",
        "Lightshield Parry",
        "Spectacular Pileup",
        "Stall Out",
        "Trade the Helm",
        "Trip Up",
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, name, Zone::Hand);
        let top = put(&mut g, p, "Shock", Zone::Library);
        let action = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, action.clone());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(&mut g, action);
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard, "{name}");
        assert_eq!(g.object_view(top).zone, Zone::Library);
        drain(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Hand, "{name}");
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn bounce_gallant_and_parry_keep_their_distinct_targets_results_and_duration() {
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Clamorous Ironclad", Zone::Battlefield);
    let bounce = put(&mut g, Player::P0, "Bounce Off", Zone::Hand);
    cast(&mut g, bounce, Some(Target::Card(vehicle)));
    drain(&mut g);
    assert_eq!(g.object_view(vehicle).zone, Zone::Hand);
    let small = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let large = put(&mut g, Player::P1, "Aegis Turtle", Zone::Battlefield);
    let strike = put(&mut g, Player::P0, "Gallant Strike", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: strike });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(small),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(large),
        },
    );
    finish_targets(&mut g);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.object_view(large).zone, Zone::Graveyard);
    let parry = put(&mut g, Player::P0, "Lightshield Parry", Zone::Hand);
    cast(&mut g, parry, Some(Target::Card(small)));
    drain(&mut g);
    assert_eq!(stats(&g, small), (4, 4));
    g.finish_cleanup();
    assert_eq!(stats(&g, small), (2, 2));
}

#[test]
fn target_discounts_are_paid_for_tapped_permanents_and_mana_value_one_only() {
    for p in [Player::P0, Player::P1] {
        for (name, target_name, mana, tapped, destination) in [
            (
                "Ride's End",
                "Clamorous Ironclad",
                [1, 0, 0, 0, 0, 1],
                true,
                Zone::Exile,
            ),
            (
                "Roadside Blowout",
                "Savannah Lions",
                [0, 1, 0, 0, 0, 0],
                false,
                Zone::Hand,
            ),
        ] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), target_name, Zone::Battlefield);
            g.state.objects[target as usize].tapped = tapped;
            g.state.players[p.index()].mana = mana;
            let source = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, Some(Target::Card(target)));
            drain(&mut g);
            assert_eq!(g.object_view(target).zone, destination, "{name}");
            assert_eq!(g.state.players[p.index()].mana, [0; 6], "{name}");
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(name == "Roadside Blowout")
            );
        }
        let mut g = scenario(p);
        put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        let source = put(&mut g, p, "Roadside Blowout", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: source });
    }
}

#[test]
fn spectral_interference_and_spell_pierce_offer_the_correct_payment_or_counter() {
    for (name, target_name, price) in [
        ("Spectral Interference", "Grizzly Bears", 4),
        ("Spell Pierce", "Divination", 2),
    ] {
        for pays in [false, true] {
            let mut g = scenario(Player::P1);
            let source = put(&mut g, Player::P1, target_name, Zone::Hand);
            cast(&mut g, source, None);
            act(&mut g, Action::Pass);
            assert_eq!(g.actor(), Some(Player::P0));
            let counter = put(&mut g, Player::P0, name, Zone::Hand);
            cast(&mut g, counter, Some(Target::Spell(source)));
            g.state.players[Player::P1.index()].mana = [0, 0, 0, 0, 0, price];
            reach_choice(&mut g);
            assert_eq!(g.actor(), Some(Player::P1));
            choose(&mut g, |a| {
                if pays {
                    matches!(a, Action::PayResolutionMana { .. })
                } else {
                    matches!(a, Action::DeclineResolutionMana)
                }
            });
            drain(&mut g);
            if target_name == "Grizzly Bears" {
                assert_eq!(
                    g.object_view(source).zone,
                    if pays {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
            } else {
                assert_eq!(
                    g.state.players[1].cards_drawn_this_turn,
                    if pays { 2 } else { 0 }
                );
            }
            assert_eq!(g.state.players[1].mana[5], if pays { 0 } else { price });
        }
    }
}

#[test]
fn stock_up_keeps_the_look_private_selects_two_and_orders_the_other_three_with_restore() {
    let p = Player::P0;
    let mut parent = scenario(p);
    let ids = [
        "Grizzly Bears",
        "Shock",
        "Island",
        "Savannah Lions",
        "Bear Cub",
    ]
    .map(|name| put(&mut parent, p, name, Zone::Library));
    let source = put(&mut parent, p, "Stock Up", Zone::Hand);
    cast(&mut parent, source, None);
    reach_choice(&mut parent);
    assert_eq!(
        parent
            .observe(p)
            .library_selection
            .as_ref()
            .unwrap()
            .remaining
            .len(),
        5
    );
    assert!(parent.observe(p.other()).library_selection.is_none());
    reject(&mut parent, Action::FinishLibrarySelection);
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let action = Action::SelectLibrary { card: ids[4] };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    for g in [&mut raw, &mut indexed] {
        act(g, Action::SelectLibrary { card: ids[3] });
        act(g, Action::FinishLibrarySelection);
        for id in [ids[0], ids[1], ids[2]] {
            act(g, Action::OrderLibraryBottom { card: id });
        }
        drain(g);
        assert_eq!(g.object_view(ids[4]).zone, Zone::Hand);
        assert_eq!(g.object_view(ids[3]).zone, Zone::Hand);
        assert_eq!(
            &g.state.players[p.index()].library[..3],
            &[ids[2], ids[1], ids[0]]
        );
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::SelectLibrary { card: ids[0] });
    assert_eq!(indexed.object_view(ids[0]).zone, Zone::Library);
    assert_eq!(raw.object_view(ids[0]).zone, Zone::Library);
    assert_eq!(raw.object_view(ids[4]).zone, Zone::Hand);
}

#[test]
fn trip_up_lets_the_owner_choose_top_or_bottom_even_under_foreign_control() {
    for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
        let mut g = scenario(Player::P0);
        let target = put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
        g.state.objects[target as usize].controller = Player::P0;
        let source = put(&mut g, Player::P0, "Trip Up", Zone::Hand);
        cast(&mut g, source, Some(Target::Card(target)));
        reach_choice(&mut g);
        assert_eq!(g.actor(), Some(Player::P1));
        let object = g.object_ref(target);
        act(
            &mut g,
            Action::ChooseLibraryDestination {
                object,
                destination,
            },
        );
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Library);
        let library = &g.state.players[1].library;
        assert_eq!(
            if destination == LibraryDestination::Top {
                library.last()
            } else {
                library.first()
            },
            Some(&target)
        );
    }
}
