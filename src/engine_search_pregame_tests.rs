//! Admitted printed fixtures exercising shared pregame and affected-player search rules.
use super::ability_timeline_tests::{checked, settle, setup};
use super::tests::spawn;
use super::*;

fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported());
    spawn(g, p, card, zone)
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
}
fn opening(start: Player) -> Game {
    Game::new(Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 71,
        starting_player: start,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap()
}
fn activate(g: &mut Game, id: CardId, ability: u16, target: Option<CardId>, payment: Mana) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(id) = target {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let actions = g.legal_actions();
        let action = if actions.contains(&Action::FinishTargets) {
            Action::FinishTargets
        } else {
            Action::PayCosts { payment }
        };
        checked(g, action);
    }
    panic!("unpaid activation");
}
fn resolve_search(g: &mut Game) {
    for _ in 0..5 {
        if matches!(g.state.pending, Pending::OptionalSearch(_)) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing search");
}
#[test]
fn opening_is_after_both_keeps_private_ordered_optional_and_exact() {
    for start in [Player::P0, Player::P1] {
        let mut g = opening(start);
        let first = fixture(&mut g, start, "Leyline Axe", Zone::Hand);
        let second = fixture(&mut g, start, "Leyline Axe", Zone::Hand);
        let other = fixture(&mut g, start.other(), "Leyline Axe", Zone::Hand);
        let first_ref = g.object_ref(first);
        reject(&mut g, Action::BeginOnBattlefield { object: first_ref });
        checked(&mut g, Action::Keep);
        assert!(matches!(g.state.pending, Pending::Mulligan(_)));
        checked(&mut g, Action::Keep);
        assert_eq!(g.state.step, Step::Pregame);
        assert!(
            !g.observe(start.other())
                .objects
                .iter()
                .any(|o| o.id == first || o.id == second)
        );
        assert!(g.observe(start.other()).actions.is_empty());
        let other_ref = g.object_ref(other);
        reject(&mut g, Action::BeginOnBattlefield { object: other_ref });
        let later = fixture(&mut g, start, "Leyline Axe", Zone::Hand);
        let later_ref = g.object_ref(later);
        reject(&mut g, Action::BeginOnBattlefield { object: later_ref });
        assert!(
            !g.observe(start.other())
                .objects
                .iter()
                .any(|o| o.id == later)
        );
        checked(&mut g, Action::BeginOnBattlefield { object: first_ref });
        assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
        reject(&mut g, Action::BeginOnBattlefield { object: first_ref });
        checked(&mut g, Action::FinishOpeningHand);
        let other_ref = g.object_ref(other);
        checked(&mut g, Action::BeginOnBattlefield { object: other_ref });
        checked(&mut g, Action::FinishOpeningHand);
        assert_eq!(g.state.step, Step::Upkeep);
        assert_eq!(g.state.objects[second as usize].zone, Zone::Hand);
        assert!(
            !g.observe(start.other())
                .objects
                .iter()
                .any(|o| o.id == second)
        );
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn opening_empty_and_declined_hands_have_same_public_phase_and_mulligan_bottoms_first() {
    for start in [Player::P0, Player::P1] {
        let mut empty = opening(start);
        checked(&mut empty, Action::Keep);
        checked(&mut empty, Action::Keep);
        assert_eq!(
            empty.legal_actions(),
            vec![Action::Concede, Action::FinishOpeningHand]
        );
        checked(&mut empty, Action::FinishOpeningHand);
        assert_eq!(
            empty.legal_actions(),
            vec![Action::Concede, Action::FinishOpeningHand]
        );
        checked(&mut empty, Action::FinishOpeningHand);
        let mut g = opening(start);
        checked(&mut g, Action::Mulligan);
        checked(&mut g, Action::Keep);
        assert!(matches!(g.state.pending, Pending::Bottom { .. }));
        let axe = fixture(&mut g, start, "Leyline Axe", Zone::Hand);
        let axe_ref = g.object_ref(axe);
        reject(&mut g, Action::BeginOnBattlefield { object: axe_ref });
        checked(&mut g, Action::Bottom { card: axe });
        checked(&mut g, Action::Keep);
        assert_eq!(
            g.legal_actions(),
            vec![Action::Concede, Action::FinishOpeningHand]
        );
        reject(&mut g, Action::BeginOnBattlefield { object: axe_ref });
        checked(&mut g, Action::FinishOpeningHand);
        checked(&mut g, Action::FinishOpeningHand);
        assert_eq!(g.state.objects[axe as usize].zone, Zone::Library);
    }
}
#[test]
fn axe_paid_equip_stats_keywords_and_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let axe = fixture(&mut g, p, "Leyline Axe", Zone::Battlefield);
        let bear = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        g.state.players[p.index()].mana = [3, 0, 0, 0, 0, 0];
        activate(&mut g, axe, 0, Some(bear), [3, 0, 0, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(
            g.keywords(bear)
                .contains(Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE))
        );
        g.move_object(axe, Zone::Graveyard);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(
            !g.keywords(bear)
                .intersects(Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE))
        );
    }
}
#[test]
fn demolition_paid_destroy_saved_controller_optional_order_private_searches_and_shuffle() {
    for p in [Player::P0, Player::P1] {
        for indestructible in [false, true] {
            let mut g = setup(p);
            let field = fixture(&mut g, p, "Demolition Field", Zone::Battlefield);
            let land = spawn(
                &mut g,
                p,
                Card::from_name("Rogue's Passage").unwrap(),
                Zone::Battlefield,
            );
            // Owner differs from affected controller; search belongs to controller.
            g.state.objects[land as usize].controller = p.other();
            if indestructible {
                g.grant_keyword_effects(&[land], g.object_ref(land), Keywords::INDESTRUCTIBLE);
            }
            g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 0];
            activate(&mut g, field, 1, Some(land), [2, 0, 0, 0, 0, 0]);
            assert_eq!(g.state.objects[field as usize].zone, Zone::Graveyard);
            // A new incarnation of the paid source cannot redirect the saved ability controller.
            g.move_object(field, Zone::Battlefield);
            g.state.objects[field as usize].controller = p.other();
            resolve_search(&mut g);
            assert_eq!(
                g.state.objects[land as usize].zone,
                if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.actor(), Some(p.other()));
            let rng = g.state.rng;
            let library = g.state.players[p.other().index()].library.clone();
            assert!(g.observe(p.other()).search_choice.is_none());
            checked(&mut g, Action::ChooseOptional { accept: false });
            assert_eq!(g.state.rng, rng);
            assert_eq!(g.state.players[p.other().index()].library, library);
            assert_eq!(g.actor(), Some(p));
            checked(&mut g, Action::ChooseOptional { accept: true });
            assert!(g.observe(p.other()).search_choice.is_none());
            assert!(g.observe(p.other()).actions.is_empty());
            let selected = g.state.players[p.index()].library[0];
            let foreign = g.state.players[p.other().index()].library[0];
            reject(&mut g, Action::ChooseSearch { card: foreign });
            checked(&mut g, Action::ChooseSearch { card: selected });
            let mut expected = g.state.players[p.index()].library.clone();
            expected.retain(|&id| id != selected);
            let mut expected_rng = rng;
            expected_rng.shuffle(&mut expected);
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.state.objects[selected as usize].zone, Zone::Battlefield);
            assert!(!g.state.objects[selected as usize].tapped);
            assert_eq!(g.state.players[p.index()].library, expected);
            assert_eq!(g.state.rng, expected_rng);
        }
    }
}
#[test]
fn demolition_failed_to_find_still_shuffles_and_illegal_target_fizzles_all_searches() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let field = fixture(&mut g, p, "Demolition Field", Zone::Battlefield);
        let land = spawn(
            &mut g,
            p.other(),
            Card::from_name("Rogue's Passage").unwrap(),
            Zone::Battlefield,
        );
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 0];
        activate(&mut g, field, 1, Some(land), [2, 0, 0, 0, 0, 0]);
        resolve_search(&mut g);
        let mut expected = g.state.players[p.other().index()].library.clone();
        let mut rng = g.state.rng;
        rng.shuffle(&mut expected);
        checked(&mut g, Action::ChooseOptional { accept: true });
        checked(&mut g, Action::FinishSearch);
        assert_eq!(g.state.players[p.other().index()].library, expected);
        assert_eq!(g.state.rng, rng);
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        let mut g = setup(p);
        let field = fixture(&mut g, p, "Demolition Field", Zone::Battlefield);
        let land = spawn(
            &mut g,
            p.other(),
            Card::from_name("Rogue's Passage").unwrap(),
            Zone::Battlefield,
        );
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 0];
        activate(&mut g, field, 1, Some(land), [2, 0, 0, 0, 0, 0]);
        g.move_object(land, Zone::Exile);
        g.move_object(land, Zone::Battlefield);
        let rng = g.state.rng;
        settle(&mut g);
        assert_eq!(g.state.rng, rng);
        assert!(g.state.resolving.is_none());
    }
}

#[test]
fn demolition_only_targets_opposing_nonbasics_and_first_search_enters_before_second() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let field = fixture(&mut g, p, "Demolition Field", Zone::Battlefield);
        let own_land = spawn(
            &mut g,
            p,
            Card::from_name("Rogue's Passage").unwrap(),
            Zone::Battlefield,
        );
        let land = spawn(
            &mut g,
            p.other(),
            Card::from_name("Rogue's Passage").unwrap(),
            Zone::Battlefield,
        );
        let basic = spawn(&mut g, p.other(), Card::Forest, Zone::Battlefield);
        let creature = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 0];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: field,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        for bad in [own_land, basic, creature, field] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        reject(
            &mut g,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 0, 0],
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [2, 0, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        resolve_search(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        let selected = g.state.players[p.other().index()].library[0];
        checked(&mut g, Action::ChooseSearch { card: selected });
        checked(&mut g, Action::FinishSearch);
        assert_eq!(g.state.objects[selected as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[selected as usize].controller, p.other());
        assert!(!g.state.objects[selected as usize].tapped);
        assert_eq!(g.actor(), Some(p));
        let rng = g.state.rng;
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert_eq!(g.state.rng, rng);
    }
}

#[test]
fn pregame_actions_do_not_run_state_based_actions_before_initial_priority() {
    for start in [Player::P0, Player::P1] {
        let mut g = opening(start);
        checked(&mut g, Action::Keep);
        checked(&mut g, Action::Keep);
        // A zero-toughness permanent is an adversarial precondition for the timing
        // boundary; pregame has no priority or state-based checks.
        let bear = spawn(&mut g, start, Card::GrizzlyBears, Zone::Battlefield);
        g.state.objects[bear as usize].counters.minus_one = 2;
        checked(&mut g, Action::FinishOpeningHand);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
        checked(&mut g, Action::FinishOpeningHand);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.step, Step::Upkeep);
    }
}

#[test]
fn declined_opening_card_is_observationally_indistinguishable_from_another_hidden_card() {
    for start in [Player::P0, Player::P1] {
        let mut plain = opening(start);
        let mut hidden = plain.clone();
        let id = hidden
            .state
            .objects
            .iter()
            .position(|o| o.owner == start && o.zone == Zone::Hand)
            .unwrap();
        hidden.state.objects[id].card = Card::from_name("Leyline Axe").unwrap();
        assert!(hidden.state.objects[id].card.is_supported());
        for action in [
            Action::Keep,
            Action::Keep,
            Action::FinishOpeningHand,
            Action::FinishOpeningHand,
        ] {
            checked(&mut plain, action.clone());
            checked(&mut hidden, action);
            assert_eq!(
                serde_json::to_value(plain.observe(start.other())).unwrap(),
                serde_json::to_value(hidden.observe(start.other())).unwrap()
            );
        }
    }
}
