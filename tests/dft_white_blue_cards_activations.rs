//! Exact paid activation, exhaust, cycling, discard-batch and linked-exile outcomes.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if idle(g) {
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
    panic!("activation resolution ceiling");
}
fn ready(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    for _ in 0..32 {
        if g.legal_actions().iter().any(&predicate) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("activation decision ceiling");
}
fn begin(g: &mut Game, source: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn finish(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main1;
    g.state.passes = 0;
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
fn detention_and_snare_exile_their_distinct_opposing_targets_until_the_source_leaves() {
    for p in [Player::P0, Player::P1] {
        for (name, target_name) in [
            ("Detention Chariot", "Grizzly Bears"),
            ("Perilous Snare", "Unstoppable Plan"),
        ] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), target_name, Zone::Battlefield);
            let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let source = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None);
            ready(&mut g, |a| {
                *a == Action::ChooseTarget {
                    target: Target::Card(target),
                }
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish(&mut g);
            drain(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
            if name == "Detention Chariot" {
                let pilot = put(&mut g, p, "Hill Giant", Zone::Battlefield);
                begin(&mut g, source, 0);
                if matches!(g.state.pending, Pending::PayCosts { .. }) {
                    pay(&mut g);
                }
                let object = g.object_ref(pilot);
                act(&mut g, Action::ChooseCrewer { object });
                act(&mut g, Action::FinishCrew);
                drain(&mut g);
                assert_eq!(stats(&g, source), (6, 6));
            } else {
                assert_eq!(g.state.players[p.index()].speed, 1);
            }
            g.move_object(source, Zone::Graveyard);
            drain(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).controller, p.other());
        }
    }
}

#[test]
fn snare_max_speed_activation_requires_four_sorcery_timing_and_a_controlled_creature_or_vehicle() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Perilous Snare", Zone::Battlefield);
    let own = put(&mut g, p, "Air Response Unit", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let action = Action::AnnounceSourceGrant {
        source: g.object_ref(source),
        ability: 0,
    };
    g.state.players[0].speed = 3;
    reject(&mut g, action.clone());
    g.state.players[0].speed = 4;
    g.state.step = Step::BeginCombat;
    reject(&mut g, action.clone());
    g.state.step = Step::Main1;
    act(&mut g, action);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    finish(&mut g);
    pay(&mut g);
    assert!(g.object_view(source).tapped);
    drain(&mut g);
    assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
    assert!(!g.is_creature(own));
}

#[test]
fn diversion_sacrifices_for_blue_and_counters_only_instant_or_sorcery_unless_three_is_paid() {
    for pays in [false, true] {
        let mut g = scenario(Player::P1);
        let unit = put(&mut g, Player::P0, "Diversion Unit", Zone::Battlefield);
        assert_eq!(stats(&g, unit), (2, 1));
        assert!(g.keywords(unit).contains(Keywords::FLYING));
        let spell = put(&mut g, Player::P1, "Divination", Zone::Hand);
        cast(&mut g, spell, None);
        act(&mut g, Action::Pass);
        g.state.players[0].mana = [0, 1, 0, 0, 0, 0];
        begin(&mut g, unit, 0);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(spell),
            },
        );
        finish(&mut g);
        pay(&mut g);
        assert_eq!(g.object_view(unit).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        g.state.players[1].mana = [0, 0, 0, 0, 0, 3];
        ready(&mut g, |a| matches!(a, Action::DeclineResolutionMana));
        choose(&mut g, |a| {
            if pays {
                matches!(a, Action::PayResolutionMana { .. })
            } else {
                matches!(a, Action::DeclineResolutionMana)
            }
        });
        drain(&mut g);
        assert_eq!(
            g.state.players[1].cards_drawn_this_turn,
            if pays { 2 } else { 0 }
        );
    }
    let mut g = scenario(Player::P1);
    let unit = put(&mut g, Player::P0, "Diversion Unit", Zone::Battlefield);
    let creature = put(&mut g, Player::P1, "Grizzly Bears", Zone::Hand);
    cast(&mut g, creature, None);
    act(&mut g, Action::Pass);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: unit,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(g.object_view(unit).zone, Zone::Battlefield);
}

#[test]
fn detentions_white_cycling_discards_before_drawing_without_its_entry_exile() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Detention Chariot", Zone::Hand);
    let target = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let top = put(&mut g, p, "Shock", Zone::Library);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 0];
    begin(&mut g, source, 1);
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.object_view(top).zone, Zone::Library);
    drain(&mut g);
    assert_eq!(g.object_view(top).zone, Zone::Hand);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn howlers_cycling_shrinks_an_opposing_uncrewed_vehicle_for_the_turn_and_draws_one() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Howler's Heavy", Zone::Hand);
    let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let vehicle = put(&mut g, p.other(), "Veloheart Bike", Zone::Battlefield);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 1];
    begin(&mut g, source, 0);
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    ready(&mut g, |a| {
        *a == Action::ChooseTarget {
            target: Target::Card(vehicle),
        }
    });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(vehicle),
        },
    );
    finish(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert!(!g.is_creature(vehicle));
    animate(&mut g, vehicle);
    assert_eq!(stats(&g, vehicle), (1, 2));
    g.finish_cleanup();
    animate(&mut g, vehicle);
    assert_eq!(stats(&g, vehicle), (4, 2));
    main(&mut g, p);
    g.state.players[0].mana = [8; 6];
    let body = put(&mut g, p, "Howler's Heavy", Zone::Hand);
    cast(&mut g, body, None);
    drain(&mut g);
    assert_eq!(stats(&g, body), (3, 4));
}

#[test]
fn mindspring_pays_chosen_x_and_tap_but_counters_each_own_merfolk_even_for_zero() {
    for x in [0, 2] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Mindspring Merfolk", Zone::Battlefield);
        let own = put(&mut g, p, "Mindspring Merfolk", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Mindspring Merfolk", Zone::Battlefield);
        let nonmerfolk = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.players[0].mana = [0, 2, 0, 0, 0, x];
        begin(&mut g, source, 0);
        act(&mut g, Action::ChooseX { value: x });
        pay(&mut g);
        assert!(g.object_view(source).tapped);
        drain(&mut g);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, u64::from(x));
        assert_eq!(stats(&g, source), (2, 2));
        for id in [source, own] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        }
        for id in [enemy, nonmerfolk] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
        }
        g.state.objects[source as usize].tapped = false;
        g.state.players[0].mana = [8; 6];
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

#[test]
fn refueler_draws_for_own_exhaust_including_itself_and_its_animation_is_permanent() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Rangers' Refueler", Zone::Battlefield);
    let pilot = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    begin(&mut g, source, 1);
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    let object = g.object_ref(pilot);
    act(&mut g, Action::ChooseCrewer { object });
    act(&mut g, Action::FinishCrew);
    drain(&mut g);
    assert_eq!(stats(&g, source), (3, 3));
    g.finish_cleanup();
    main(&mut g, p);
    assert!(!g.is_creature(source));
    g.state.players[0].mana = [0, 0, 0, 0, 0, 4];
    begin(&mut g, source, 0);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(stats(&g, source), (4, 4));
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
    g.state.players[0].mana = [8; 6];
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let engineer = put(&mut g, p, "Skystreak Engineer", Zone::Battlefield);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 4];
    begin(&mut g, engineer, 0);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(stats(&g, engineer), (3, 5));
    assert!(g.keywords(engineer).contains(Keywords::FLYING));
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    g.state.players[0].mana = [8; 6];
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: engineer,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let other = put(&mut g, p.other(), "Skystreak Engineer", Zone::Battlefield);
    act(&mut g, Action::Pass);
    // Cleanup cleared both players' pools before the opponent's activation.
    g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 4];
    begin(&mut g, other, 0);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    g.finish_cleanup();
    assert!(g.is_creature(source));
    assert_eq!(stats(&g, source), (4, 4));
}

#[test]
fn keen_draws_then_discards_and_adds_one_counter_with_vigilance_and_exhaust_consumed() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Keen Buccaneer", Zone::Battlefield);
    let discard = put(&mut g, p, "Shock", Zone::Hand);
    let top = put(&mut g, p, "Forest", Zone::Library);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 1];
    begin(&mut g, source, 0);
    pay(&mut g);
    ready(&mut g, |a| matches!(a, Action::Discard { .. }));
    assert_eq!(g.object_view(top).zone, Zone::Hand);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    act(&mut g, Action::Discard { card: discard });
    drain(&mut g);
    assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(stats(&g, source), (3, 4));
    assert!(g.keywords(source).contains(Keywords::VIGILANCE));
    g.state.players[0].mana = [8; 6];
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
fn scrounging_skyray_counts_a_real_two_card_discard_batch_and_its_cycling_discard() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Scrounging Skyray", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Scrounging Skyray", Zone::Battlefield);
    let kiora = put(&mut g, p, "Kiora, the Rising Tide", Zone::Hand);
    cast(&mut g, kiora, None);
    ready(&mut g, |a| matches!(a, Action::Discard { .. }));
    choose(&mut g, |a| matches!(a, Action::Discard { .. }));
    choose(&mut g, |a| matches!(a, Action::Discard { .. }));
    drain(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(stats(&g, source), (3, 4));
    assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
    assert!(g.keywords(source).contains(Keywords::FLYING));
    let cycling = put(&mut g, p, "Scrounging Skyray", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    begin(&mut g, cycling, 0);
    pay(&mut g);
    assert_eq!(g.object_view(cycling).zone, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 3);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn repurposing_bay_pays_another_artifact_and_searches_exactly_one_more_mana_value() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Repurposing Bay", Zone::Battlefield);
        let material = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
        let valid = put(&mut g, p, "Air Response Unit", Zone::Library);
        let wrong = put(&mut g, p, "Swiftfoot Boots", Zone::Library);
        let spell = put(&mut g, p, "Divination", Zone::Library);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let start = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.step = Step::BeginCombat;
        reject(&mut g, start.clone());
        g.state.step = Step::Main1;
        act(&mut g, start);
        let object = g.object_ref(source);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(material);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.object_view(material).zone, Zone::Graveyard);
        ready(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
        for card in [wrong, spell] {
            reject(&mut g, Action::ChooseSearch { card });
        }
        act(&mut g, Action::ChooseSearch { card: valid });
        act(&mut g, Action::FinishSearch);
        drain(&mut g);
        assert_eq!(g.object_view(valid).zone, Zone::Battlefield);
        assert!(!g.object_view(valid).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn riverchurn_targets_each_player_and_uses_each_separate_current_graveyard_for_exhaust() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Riverchurn Monument", Zone::Battlefield);
    for _ in 0..3 {
        put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    }
    put(&mut g, p.other(), "Grizzly Bears", Zone::Graveyard);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    begin(&mut g, source, 0);
    for player in [p, p.other()] {
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(player),
            },
        );
    }
    finish(&mut g);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].library.len(), 6);
    assert_eq!(g.state.players[1].library.len(), 6);
    g.state.objects[source as usize].tapped = false;
    g.state.players[0].mana = [0, 2, 0, 0, 0, 2];
    begin(&mut g, source, 1);
    for player in [p, p.other()] {
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(player),
            },
        );
    }
    finish(&mut g);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].library.len(), 1);
    assert_eq!(g.state.players[1].library.len(), 3);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    g.state.objects[source as usize].tapped = false;
    g.state.players[0].mana = [8; 6];
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    let other = put(&mut g, p, "Riverchurn Monument", Zone::Battlefield);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    begin(&mut g, other, 0);
    act(&mut g, Action::FinishTargets);
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].library.len(), 1);
    assert_eq!(g.state.players[1].library.len(), 3);
}
