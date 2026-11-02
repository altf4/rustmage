//! Whole-card outcomes for the black/red creature and activation family.
mod support;
use rustmage::Card;
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
fn activate(g: &mut Game, card: CardId, ability: u16) {
    act(g, activation(card, ability));
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
}
fn wait_library(g: &mut Game) {
    for _ in 0..16 {
        g.prepare_triggers();
        if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("surveil choice missing");
}
fn wait_effect(g: &mut Game) {
    for _ in 0..16 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("discard choice missing");
}
fn arrange(g: &mut Game, card: CardId, destination: LibraryDestination) {
    act(g, Action::ArrangeLibrary { card, destination });
}
fn assert_spent(g: &Game, source: CardId, ability: u16) {
    assert!(
        !g.legal_actions().contains(&activation(source, ability)),
        "used exhaust slot {ability}"
    );
    assert_ne!(g.object_view(source).used_abilities & (1 << ability), 0);
}

#[test]
fn kalakscion_and_tyrox_cast_with_printed_stats_and_are_actually_abilityless() {
    for p in [Player::P0, Player::P1] {
        for (name, mana, expected) in [
            ("Kalakscion, Hunger Tyrant", [0, 0, 2, 0, 0, 1], (7, 2)),
            ("Tyrox, Saurid Tyrant", [0, 0, 0, 1, 0, 1], (4, 1)),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = mana;
            cast(&mut g, source, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(stats(&g, source), expected);
            assert!(g.object_view(source).card.is_legendary());
            assert_eq!(g.keywords(source), Keywords::NONE);
            assert!(!Card::from_name(name).unwrap().has_rules_abilities());
            assert!(g.object_search_filter_matches(SearchFilter::NoAbilities, source));
            let contrast = put(&mut g, p, "Engine Rat", Zone::Battlefield);
            assert!(!g.object_search_filter_matches(SearchFilter::NoAbilities, contrast));
        }
    }
}

#[test]
fn engine_rat_pays_five_generic_and_black_for_repeatable_opponent_life_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Engine Rat", Zone::Battlefield);
        assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
        reject(&mut g, activation(source, 0));
        for expected in [18, 16] {
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 5];
            activate(&mut g, source, 0);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(!g.object_view(source).tapped);
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, expected);
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}

#[test]
fn engine_rats_one_damage_is_lethal_to_a_larger_creature() {
    let mut g = scenario(Player::P0);
    let rat = put(&mut g, Player::P0, "Engine Rat", Zone::Battlefield);
    let target = put(&mut g, Player::P1, "Aegis Turtle", Zone::Battlefield);
    g.apply_damage_batch(&[DamageEvent {
        source: g.damage_source(rat),
        target: Target::Card(target),
        amount: 1,
        combat: true,
        preventable: true,
    }]);
    g.advance_replacements(false);
    g.state_based_actions();
    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    assert_eq!(g.object_view(rat).zone, Zone::Battlefield);
}

#[test]
fn thunderhead_gunner_discards_as_cost_obeys_sorcery_and_once_each_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Thunderhead Gunner", Zone::Battlefield);
        assert!(g.keywords(source).contains(Keywords::REACH));
        reject(&mut g, activation(source, 0));
        let discard = put(&mut g, p, "Mountain", Zone::Hand);
        let other = put(&mut g, p, "Forest", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Mountain", Zone::Hand);
        g.state.step = Step::Upkeep;
        reject(&mut g, activation(source, 0));
        g.state.step = Step::Main1;
        act(&mut g, activation(source, 0));
        let wrong = g.object_ref(foreign);
        reject(&mut g, Action::ChooseAdditionalCost { object: wrong });
        let selected = g.object_ref(discard);
        act(&mut g, Action::ChooseAdditionalCost { object: selected });
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.object_view(other).zone, Zone::Hand);
        assert_eq!(g.object_view(foreign).zone, Zone::Hand);
        reject(&mut g, activation(source, 0));
        g.finish_cleanup();
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        assert!(
            g.legal_actions().contains(&activation(source, 0)),
            "available on a later turn"
        );
    }
}

#[test]
fn wretched_doll_pays_black_and_tap_then_privately_surveils_one() {
    for p in [Player::P0, Player::P1] {
        for grave in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Wretched Doll", Zone::Battlefield);
            let top = put(&mut g, p, "Shock", Zone::Library);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            g.state.objects[source as usize].summoning_sick = true;
            reject(&mut g, activation(source, 0));
            g.state.objects[source as usize].summoning_sick = false;
            activate(&mut g, source, 0);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.object_view(source).tapped);
            wait_library(&mut g);
            assert_eq!(
                g.observe(p)
                    .library_choice
                    .as_ref()
                    .unwrap()
                    .remaining
                    .len(),
                1
            );
            assert!(g.observe(p.other()).library_choice.is_none());
            assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
            arrange(
                &mut g,
                top,
                if grave {
                    LibraryDestination::Graveyard
                } else {
                    LibraryDestination::Top
                },
            );
            settle(&mut g);
            assert_eq!(
                g.object_view(top).zone,
                if grave {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
            assert!(g.state.players[p.index()].scried_or_surveilled_this_turn);
        }
    }
}

#[test]
fn wreckage_wickerfolk_enters_flying_and_surveils_two_with_real_reordering() {
    for p in [Player::P0, Player::P1] {
        for grave in [false, true] {
            let mut g = scenario(p);
            let second = put(&mut g, p, "Shock", Zone::Library);
            let first = put(&mut g, p, "Divination", Zone::Library);
            let source = put(&mut g, p, "Wreckage Wickerfolk", Zone::Hand);
            cast(&mut g, source, None);
            wait_library(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert!(g.keywords(source).contains(Keywords::FLYING));
            assert_eq!(stats(&g, source), (1, 3));
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
            arrange(
                &mut g,
                second,
                if grave {
                    LibraryDestination::Graveyard
                } else {
                    LibraryDestination::Top
                },
            );
            arrange(&mut g, first, LibraryDestination::Top);
            settle(&mut g);
            assert_eq!(
                g.object_view(second).zone,
                if grave {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
            assert_eq!(
                g.state.players[p.index()].library.last(),
                Some(if grave { &first } else { &second })
            );
        }
    }
}

#[test]
fn deathless_and_dynamite_contribute_two_extra_power_without_changing_stats() {
    for p in [Player::P0, Player::P1] {
        for (name, printed) in [("Deathless Pilot", (2, 2)), ("Dynamite Diver", (1, 1))] {
            for crew in [false, true] {
                let mut g = scenario(p);
                let pilot = put(&mut g, p, name, Zone::Battlefield);
                let source = put(
                    &mut g,
                    p,
                    if crew {
                        "Clamorous Ironclad"
                    } else {
                        "Dracosaur Auxiliary"
                    },
                    Zone::Battlefield,
                );
                activate(&mut g, source, 0);
                reject(
                    &mut g,
                    if crew {
                        Action::FinishCrew
                    } else {
                        Action::FinishSaddle
                    },
                );
                let object = g.object_ref(pilot);
                act(
                    &mut g,
                    if crew {
                        Action::ChooseCrewer { object }
                    } else {
                        Action::ChooseSaddler { object }
                    },
                );
                act(
                    &mut g,
                    if crew {
                        Action::FinishCrew
                    } else {
                        Action::FinishSaddle
                    },
                );
                settle(&mut g);
                assert!(g.object_view(pilot).tapped);
                assert_eq!(stats(&g, pilot), printed);
                if crew {
                    assert!(g.is_creature(source));
                    assert_eq!(stats(&g, source), (6, 3));
                } else {
                    assert!(g.object_view(source).saddled);
                }
            }
        }
    }
}

#[test]
fn deathless_pilot_returns_its_exact_owned_graveyard_card_and_not_a_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Deathless Pilot", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        activate(&mut g, source, 0);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let mut stale = g.clone();
        stale.move_object(source, Zone::Hand);
        stale.move_object(source, Zone::Graveyard);
        settle(&mut g);
        settle(&mut stale);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
        assert_eq!(stale.object_view(source).zone, Zone::Graveyard);
        assert_ne!(g.object_ref(source), stale.object_ref(source));
    }
}

#[test]
fn dynamite_divers_death_damage_has_all_target_classes_and_survives_source_movement() {
    for kind in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Dynamite Diver", Zone::Battlefield);
        let target = match kind {
            0 => Target::Player(p.other()),
            1 => Target::Card(put(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield)),
            _ => {
                let id = put(
                    &mut g,
                    p.other(),
                    "Chandra, Spark Hunter",
                    Zone::Battlefield,
                );
                g.state.objects[id as usize].counters.loyalty = 5;
                Target::Card(id)
            }
        };
        g.move_object(source, Zone::Graveyard);
        for _ in 0..12 {
            g.prepare_triggers();
            let action = Action::ChooseTarget { target };
            if g.legal_actions().contains(&action) {
                act(&mut g, action);
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        match target {
            Target::Player(victim) => assert_eq!(g.state.players[victim.index()].life, 19),
            Target::Card(id) if kind == 1 => assert_eq!(g.object_view(id).damage, 1),
            Target::Card(id) => assert_eq!(g.object_view(id).counters.loyalty, 4),
            _ => unreachable!(),
        }
    }
}

#[test]
fn pacesetter_and_prowcatcher_pay_distinct_exhaust_costs_and_keep_their_counters() {
    for (name, generic, counters, expected) in [
        ("Pacesetter Paragon", 2, 1, (3, 4)),
        ("Prowcatcher Specialist", 3, 2, (4, 3)),
    ] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, name, Zone::Battlefield);
        g.state.players[0].mana = [0, 0, 0, 0, 0, generic + 1];
        reject(&mut g, activation(source, 0));
        g.state.players[0].mana = [0, 0, 0, 1, 0, generic];
        activate(&mut g, source, 0);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), counters);
        assert_eq!(stats(&g, source), expected);
        assert_eq!(
            g.keywords(source).contains(Keywords::DOUBLE_STRIKE),
            name == "Pacesetter Paragon"
        );
        assert_eq!(
            g.keywords(source).contains(Keywords::HASTE),
            name == "Prowcatcher Specialist"
        );
        assert_spent(&g, source, 0);
        g.finish_cleanup();
        assert_eq!(stats(&g, source), expected);
        assert!(!g.keywords(source).contains(Keywords::DOUBLE_STRIKE));
        assert_eq!(
            g.keywords(source).contains(Keywords::HASTE),
            name == "Prowcatcher Specialist"
        );
        g.state.active = Player::P0;
        g.state.priority = Player::P0;
        g.state.pending = Pending::Priority;
        g.state.step = Step::Main1;
        g.state.players[0].mana = [8; 6];
        assert_spent(&g, source, 0);
    }
}

#[test]
fn draconautics_exhaust_slots_are_independent_and_haste_freezes_only_current_others() {
    for p in [Player::P0, Player::P1] {
        for token_first in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Draconautics Engineer", Zone::Battlefield);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let opponent = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
            let before = g.state.objects.len();
            for ability in if token_first { [1, 0] } else { [0, 1] } {
                g.state.players[p.index()].mana = if ability == 0 {
                    [0, 0, 0, 1, 0, 0]
                } else {
                    [0, 0, 0, 1, 0, 3]
                };
                activate(&mut g, source, ability);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                settle(&mut g);
                assert_spent(&g, source, ability);
            }
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .skip(before)
                .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token() && o.controller == p)
                .map(|(id, _)| id as CardId)
                .collect();
            assert_eq!(tokens.len(), 1);
            let token = tokens[0];
            assert_eq!(stats(&g, token), (4, 4));
            assert_eq!(g.object_view(token).colors, 1 << 3);
            assert!(g.has_subtype(token, "Dinosaur") && g.has_subtype(token, "Dragon"));
            assert!(g.keywords(token).contains(Keywords::FLYING));
            assert!(!g.object_view(token).tapped);
            assert_eq!(g.keywords(token).contains(Keywords::HASTE), token_first);
            assert!(g.keywords(own).contains(Keywords::HASTE));
            assert!(!g.keywords(source).contains(Keywords::HASTE));
            assert!(!g.keywords(opponent).contains(Keywords::HASTE));
            assert!(!g.keywords(vehicle).contains(Keywords::HASTE));
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
            let later = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            assert!(!g.keywords(later).contains(Keywords::HASTE));
            g.finish_cleanup();
            assert!(!g.keywords(own).contains(Keywords::HASTE));
            assert_eq!(stats(&g, source), (3, 3));
        }
    }
}

#[test]
fn greasewrench_goblin_discards_zero_one_or_two_then_draws_exactly_that_many() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Greasewrench Goblin", Zone::Battlefield);
            let cards: Vec<_> = (0..3)
                .map(|_| put(&mut g, p, "Mountain", Zone::Hand))
                .collect();
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
            activate(&mut g, source, 0);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            wait_effect(&mut g);
            for &card in cards.iter().take(count) {
                act(&mut g, Action::Discard { card });
            }
            if count < 2 {
                act(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                count as u64
            );
            assert_eq!(
                cards
                    .iter()
                    .filter(|&&id| g.object_view(id).zone == Zone::Graveyard)
                    .count(),
                count
            );
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
            assert_eq!(stats(&g, source), (3, 2));
            assert_spent(&g, source, 0);
        }
    }
}

#[test]
fn greasewrench_pending_discard_has_private_owned_raw_index_and_restore_branches() {
    let p = Player::P1;
    let mut parent = scenario(p);
    let source = put(&mut parent, p, "Greasewrench Goblin", Zone::Battlefield);
    let card = put(&mut parent, p, "Mountain", Zone::Hand);
    activate(&mut parent, source, 0);
    wait_effect(&mut parent);
    assert!(
        !parent
            .observe(p.other())
            .objects
            .iter()
            .any(|o| o.id == card)
    );
    let saved = parent.snapshot();
    let action = Action::Discard { card };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let at = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    act(&mut raw, action);
    indexed.apply_index(p, indexed.revision(), at).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    drop(parent);
    act(&mut indexed, Action::DeclineEffectPayment);
    act(&mut raw, Action::DeclineEffectPayment);
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.object_view(card).zone, Zone::Graveyard);
    assert_eq!(indexed.object_view(card).zone, Zone::Hand);
    assert_eq!(raw.state.players[p.index()].cards_drawn_this_turn, 1);
    assert_eq!(indexed.state.players[p.index()].cards_drawn_this_turn, 0);
    assert_eq!(raw.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(indexed.counter_count(source, CounterKind::PlusOne), 1);
}
