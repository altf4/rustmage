//! Paid machine definitions: chosen types, XX, exhaust, restricted mana and attachments.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn advance(g: &mut Game) {
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
fn drain(g: &mut Game) {
    for _ in 0..96 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        advance(g);
    }
    panic!("bounded machine resolution");
}
fn reach(g: &mut Game, wanted: impl Fn(&Action) -> bool) {
    for _ in 0..64 {
        if g.legal_actions().iter().any(&wanted) {
            return;
        }
        advance(g);
    }
    panic!("bounded machine choice");
}
fn crew(g: &mut Game, source: CardId, ability: u16, members: &[CardId]) {
    act(g, activation(source, ability));
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
    for &id in members {
        let object = g.object_ref(id);
        act(g, Action::ChooseCrewer { object });
    }
    act(g, Action::FinishCrew);
    drain(g);
}
fn x_cast(g: &mut Game, source: CardId, value: u32) {
    act(g, Action::AnnounceCast { card: source });
    act(g, Action::ChooseX { value });
    pay(g);
    drain(g);
}

#[test]
fn lifecraft_actual_entry_choice_changes_only_own_creature_vehicles_and_other_chosen_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let source = put(&mut g, p, "Lifecraft Engine", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::ChooseEntryValue { .. }));
        assert_eq!(g.object_view(source).zone, Zone::Stack);
        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(CreatureType::from_name("Bear").unwrap()),
            },
        );
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, bear), (3, 3));
        assert_eq!(stats(&g, foreign), (2, 2));
        assert!(!g.has_subtype(vehicle, "Bear"));
        crew(&mut g, source, 0, &[bear]);
        assert_eq!(
            stats(&g, source),
            (4, 4),
            "source is excluded from its own bonus"
        );
        assert!(g.has_subtype(source, "Bear"));
        // A newly cast Vehicle can crew; it does not tap itself as an activation cost.
        crew(&mut g, vehicle, 0, &[source]);
        assert!(g.has_subtype(vehicle, "Bear"));
        assert_eq!(stats(&g, vehicle), (7, 4));
        g.state.objects[source as usize].controller = p.other();
        assert_eq!(stats(&g, bear), (2, 2));
        assert_eq!(stats(&g, foreign), (3, 3));
        assert_eq!(stats(&g, vehicle), (6, 3));
        assert!(!g.has_subtype(vehicle, "Bear"));
        g.move_object(source, Zone::Graveyard);
        assert_eq!(stats(&g, foreign), (2, 2));
    }
}

#[test]
fn marketback_pays_twice_x_adds_repeatable_counters_and_draws_exact_death_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Marketback Walker", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
        x_cast(&mut g, source, 3);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, source), (3, 3));
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 3);
        for expected in [4, 5] {
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            act(&mut g, activation(source, 0));
            pay(&mut g);
            drain(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), expected);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 5);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
    }
}

#[test]
fn marketback_zero_x_dies_without_counters_and_draws_zero() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Marketback Walker", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    x_cast(&mut g, source, 0);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
}

#[test]
fn pathcruiser_actual_entry_searches_only_basic_lands_privately_and_shuffles_even_when_declined() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = scenario(p);
            let nonbasic = put(&mut g, p, "Night Market", Zone::Library);
            let basic = put(&mut g, p, "Island", Zone::Library);
            let source = put(&mut g, p, "Marshals' Pathcruiser", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            cast(&mut g, source, None);
            reach(&mut g, |a| matches!(a, Action::FinishSearch));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.observe(p).search_choice.as_ref().unwrap().reveal);
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: nonbasic });
            let rng = g.state.rng;
            if accept {
                act(&mut g, Action::ChooseSearch { card: basic });
            }
            act(&mut g, Action::FinishSearch);
            drain(&mut g);
            assert_ne!(g.state.rng, rng);
            assert_eq!(
                g.object_view(basic).zone,
                if accept { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(g.object_view(nonbasic).zone, Zone::Library);
            assert!(!g.is_creature(source));
        }
    }
}

#[test]
fn pathcruiser_crew_five_is_temporary_while_five_color_exhaust_is_permanent_and_once() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Marshals' Pathcruiser", Zone::Battlefield);
    let three = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let two = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    act(&mut g, activation(source, 1));
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    let object = g.object_ref(three);
    act(&mut g, Action::ChooseCrewer { object });
    reject(&mut g, Action::FinishCrew);
    let object = g.object_ref(two);
    act(&mut g, Action::ChooseCrewer { object });
    act(&mut g, Action::FinishCrew);
    drain(&mut g);
    assert_eq!(stats(&g, source), (6, 5));
    g.finish_cleanup();
    assert!(!g.is_creature(source));
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.players[0].mana = [0, 0, 0, 0, 0, 5];
    reject(&mut g, activation(source, 0));
    g.state.players[0].mana = [1, 1, 1, 1, 1, 0];
    act(&mut g, activation(source, 0));
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(stats(&g, source), (8, 7));
    g.finish_cleanup();
    assert_eq!(stats(&g, source), (8, 7));
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.players[0].mana = [1, 1, 1, 1, 1, 0];
    reject(&mut g, activation(source, 0));
}

#[test]
fn pit_actual_body_defender_and_two_mana_pay_abilities_but_cannot_cast_spells() {
    let p = Player::P0;
    let mut g = scenario(p);
    let pit = put(&mut g, p, "Pit Automaton", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    cast(&mut g, pit, None);
    drain(&mut g);
    assert_eq!(stats(&g, pit), (0, 4));
    assert!(g.keywords(pit).contains(Keywords::DEFENDER));
    let walker = put(&mut g, p, "Marketback Walker", Zone::Battlefield);
    g.add_counter(walker, CounterKind::PlusOne, 1);
    g.state.objects[pit as usize].summoning_sick = false;
    let mana = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::ActivateMana{card,ability:0,..} if *card==pit))
        .unwrap();
    g.state.objects[pit as usize].summoning_sick = true;
    reject(&mut g, mana.clone());
    g.state.objects[pit as usize].summoning_sick = false;
    act(&mut g, mana);
    let spell = put(&mut g, p, "Swiftfoot Boots", Zone::Hand);
    reject(&mut g, Action::AnnounceCast { card: spell });
    g.state.players[0].mana[5] += 2;
    act(&mut g, activation(walker, 0));
    pay(&mut g);
    drain(&mut g);
    assert_eq!(stats(&g, walker), (2, 2));
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert!(g.object_view(pit).tapped);
}

#[test]
fn pit_delayed_next_exhaust_survives_source_departure_ignores_ordinary_activations_and_is_consumed_once()
 {
    let p = Player::P0;
    let mut g = scenario(p);
    let pit = put(&mut g, p, "Pit Automaton", Zone::Battlefield);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    act(&mut g, activation(pit, 1));
    pay(&mut g);
    drain(&mut g);
    assert!(g.object_view(pit).tapped);
    g.move_object(pit, Zone::Graveyard);
    let walker = put(&mut g, p, "Marketback Walker", Zone::Battlefield);
    g.add_counter(walker, CounterKind::PlusOne, 1);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 4];
    act(&mut g, activation(walker, 0));
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.counter_count(walker, CounterKind::PlusOne), 2);
    for expected in [4, 2] {
        let expert = put(&mut g, p, "Afterburner Expert", Zone::Battlefield);
        g.state.players[0].mana = [0, 0, 0, 0, 2, 2];
        act(&mut g, activation(expert, 0));
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.counter_count(expert, CounterKind::PlusOne), expected);
    }
}

#[test]
fn pit_delayed_exhaust_callback_expires_at_cleanup() {
    let p = Player::P0;
    let mut g = scenario(p);
    let pit = put(&mut g, p, "Pit Automaton", Zone::Battlefield);
    act(&mut g, activation(pit, 1));
    pay(&mut g);
    drain(&mut g);
    g.finish_cleanup();
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.players[0].mana = [0, 0, 0, 0, 2, 2];
    let expert = put(&mut g, p, "Afterburner Expert", Zone::Battlefield);
    act(&mut g, activation(expert, 0));
    pay(&mut g);
    drain(&mut g);
    assert_eq!(g.counter_count(expert, CounterKind::PlusOne), 2);
}

fn lotus_begin(g: &mut Game, source: CardId, target: Player) {
    act(g, activation(source, 0));
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn sacrifice(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseAdditionalCost { object });
}
fn lotus_finish(g: &mut Game) {
    act(g, Action::FinishAdditionalCostGroup);
    pay(g);
    reach(g, |a| matches!(a, Action::ChooseMana { .. }));
}

#[test]
fn radiant_lotus_paid_entry_and_ranged_self_sacrifice_use_stack_and_controller_chooses_target_players_color()
 {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = scenario(p);
            let lotus = put(&mut g, p, "Radiant Lotus", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
            cast(&mut g, lotus, None);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let extra = put(&mut g, p, "Swiftfoot Boots", Zone::Battlefield);
            let foreign = put(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.players[p.other().index()].mana = [0; 6];
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a,Action::ActivateMana{card,..} if *card==lotus))
            );
            lotus_begin(&mut g, lotus, p.other());
            reject(&mut g, Action::FinishAdditionalCostGroup);
            for invalid in [foreign, bear] {
                let object = g.object_ref(invalid);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
            sacrifice(&mut g, lotus);
            sacrifice(&mut g, extra);
            let object = g.object_ref(extra);
            reject(&mut g, Action::ChooseAdditionalCost { object });
            act(&mut g, Action::FinishAdditionalCostGroup);
            pay(&mut g);
            assert_eq!(g.object_view(lotus).zone, Zone::Graveyard);
            assert_eq!(g.object_view(extra).zone, Zone::Graveyard);
            assert!(!g.state.stack.is_empty());
            reach(&mut g, |a| matches!(a, Action::ChooseMana { .. }));
            assert_eq!(g.actor(), Some(p));
            reject(
                &mut g,
                Action::ChooseMana {
                    production: [0, 0, 0, 0, 0, 6],
                },
            );
            let mut production = [0; 6];
            production[color] = 6;
            act(&mut g, Action::ChooseMana { production });
            drain(&mut g);
            assert_eq!(g.state.players[p.other().index()].mana, production);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn radiant_lotus_cost_selection_has_independent_owned_raw_index_restore_after_parent_disposal() {
    let p = Player::P0;
    let mut parent = scenario(p);
    let lotus = put(&mut parent, p, "Radiant Lotus", Zone::Battlefield);
    let extra = put(&mut parent, p, "Swiftfoot Boots", Zone::Battlefield);
    parent.state.players[0].mana = [0; 6];
    parent.state.players[1].mana = [0; 6];
    lotus_begin(&mut parent, lotus, p);
    sacrifice(&mut parent, lotus);
    let object = parent.object_ref(extra);
    let selection = Action::ChooseAdditionalCost { object };
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut restored = parent.clone();
    restored.restore(&saved).unwrap();
    drop(parent);
    act(&mut raw, selection.clone());
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == selection)
        .unwrap();
    indexed.apply_index(p, indexed.revision(), index).unwrap();
    for g in [&mut raw, &mut indexed, &mut restored] {
        lotus_finish(g);
        let amount = if g.object_view(extra).zone == Zone::Graveyard {
            6
        } else {
            3
        };
        act(
            g,
            Action::ChooseMana {
                production: [0, amount, 0, 0, 0, 0],
            },
        );
        drain(g);
        assert_eq!(g.state.players[0].mana, [0, amount, 0, 0, 0, 0]);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(restored.object_view(extra).zone, Zone::Battlefield);
    assert_eq!(raw.object_view(extra).zone, Zone::Graveyard);
}

#[test]
fn rover_equips_only_own_creature_at_sorcery_speed_then_crews_as_double_striker_and_detaches() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let rover = put(&mut g, p, "Rover Blades", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        cast(&mut g, rover, None);
        drain(&mut g);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        g.state.step = Step::Upkeep;
        reject(&mut g, activation(rover, 0));
        g.state.step = Step::Main1;
        act(&mut g, activation(rover, 0));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.state.objects[rover as usize].attached_to,
            Some(g.object_ref(bear))
        );
        assert!(g.keywords(bear).contains(Keywords::DOUBLE_STRIKE));
        crew(&mut g, rover, 1, &[bear]);
        assert_eq!(stats(&g, rover), (2, 2));
        assert!(g.keywords(rover).contains(Keywords::DOUBLE_STRIKE));
        assert_eq!(g.state.objects[rover as usize].attached_to, None);
        assert!(!g.keywords(bear).contains(Keywords::DOUBLE_STRIKE));
        g.finish_cleanup();
        assert!(!g.is_creature(rover));
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        act(&mut g, activation(rover, 0));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        drain(&mut g);
        assert!(g.keywords(bear).contains(Keywords::DOUBLE_STRIKE));
    }
}
