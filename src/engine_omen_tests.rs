//! Native adverse contracts; paid public and Standard games are separate gates.
use super::tests::{scenario, spawn};
use super::*;
const NAMES: [&str; 9] = [
    "Dirgur Island Dragon // Skimming Strike",
    "Riling Dawnbreaker // Signaling Roar",
    "Sagu Wildling // Roost Seek",
    "Disruptive Stormbrood // Petty Revenge",
    "Twinmaw Stormbrood // Charring Bite",
    "Marang River Regent // Coil and Catch",
    "Purging Stormbrood // Absorb Essence",
    "Runescale Stormbrood // Chilling Screech",
    "Stormshriek Feral // Flush Out",
];
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [30; 6];
    for owner in [p, p.other()] {
        spawn(
            &mut g,
            owner,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
    }
    g
}
fn act(g: &mut Game, a: Action) {
    g.apply(g.actor().unwrap(), a).unwrap();
}
fn settle(g: &mut Game) {
    for _ in 0..180 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::FinishSearch | Action::FinishTargets))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        act(g, a);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn stack(g: &mut Game, id: CardId) {
    act(g, Action::AnnounceOmen { card: id });
    for _ in 0..60 {
        if g.state.pending == Pending::Priority && g.state.stack.iter().any(|s| s.card == id) {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        act(g, a);
    }
    panic!("announcement");
}
#[test]
fn whole_nine_pairs_paid_selected_traits_shuffle_one_physical_source_and_never_grant() {
    for p in [Player::P0, Player::P1] {
        for name in NAMES {
            let mut g = setup(p);
            let card = Card::from_name(name).unwrap();
            let inset = card.omen_spell().unwrap();
            assert_eq!(inset.omen_parent(), Some(card));
            assert_eq!(card.adventure_spell(), None);
            let id = spawn(&mut g, p, card, Zone::Hand);
            if name == NAMES[7] {
                let response = spawn(&mut g, p.other(), Card::Shock, Zone::Hand);
                g.state.players[p.other().index()].mana = [30; 6];
                g.state.priority = p.other();
                super::tests::cast(&mut g, response, Some(Target::Player(p)));
                act(&mut g, Action::Pass);
            }
            let count = g.state.objects.len();
            stack(&mut g, id);
            let spell = g.state.stack.iter().find(|s| s.card == id).unwrap();
            assert_eq!(spell.source_card, inset);
            assert_eq!(spell.cast.unwrap().face, CastFace::Omen);
            assert_eq!(spell.cast.unwrap().method, CastMethod::Normal);
            assert_eq!(spell.mana_value(), inset.fixed_mana_value());
            assert!(g.has_subtype(id, "Omen"));
            assert!(!g.is_creature(id));
            let saved = g.snapshot();
            let mut child = {
                let parent = g.clone();
                let c = parent.clone();
                drop(parent);
                c
            };
            settle(&mut g);
            settle(&mut child);
            assert_eq!(g.snapshot(), child.snapshot());
            let done = g.snapshot();
            g.restore(&saved).unwrap();
            settle(&mut g);
            assert_eq!(done, g.snapshot());
            assert_eq!(g.state.objects[id as usize].zone, Zone::Library);
            assert_eq!(g.state.objects[id as usize].card, card);
            assert_eq!(
                g.state.players[p.index()]
                    .library
                    .iter()
                    .filter(|&&x| x == id)
                    .count(),
                1
            );
            assert!(g.state.adventures.is_none());
            assert!(g.state.exile_permissions.is_empty());
            assert_eq!(
                g.state.objects.len(),
                count + usize::from(name.starts_with("Riling"))
            );
            assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == id));
            assert!(
                Game::new(Config {
                    decks: [vec![inset; 60], vec![Card::Forest; 60]],
                    seed: 0,
                    starting_player: p,
                    deck_rules: DeckRules::Freeform
                })
                .is_err()
            );
        }
    }
}
#[test]
fn search_shuffle_then_actual_omen_completion_shuffle_preserve_both_rng_steps() {
    for p in [Player::P0, Player::P1] {
        for select in [false, true] {
            let mut g = setup(p);
            let card = Card::from_name(NAMES[2]).unwrap();
            let id = spawn(&mut g, p, card, Zone::Hand);
            stack(&mut g, id);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            assert_eq!(g.observe(p).decision, "library_search");
            let selected = if select {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ChooseSearch { .. }))
                    .unwrap();
                let Action::ChooseSearch { card } = a else {
                    unreachable!()
                };
                act(&mut g, a);
                Some(card)
            } else {
                None
            };
            let mut rng = g.state.rng;
            let mut expected = g.state.players[p.index()].library.clone();
            if let Some(card) = selected {
                expected.retain(|&x| x != card);
            }
            rng.shuffle(&mut expected);
            expected.push(id);
            rng.shuffle(&mut expected);
            let saved = g.snapshot();
            let rev = g.revision();
            act(&mut g, Action::FinishSearch);
            assert_eq!(g.state.players[p.index()].library, expected);
            assert_eq!(g.state.rng, rng);
            assert!(g.state.adventures.is_none());
            let done = g.snapshot();
            g.restore(&saved).unwrap();
            assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
            act(&mut g, Action::FinishSearch);
            assert_eq!(g.snapshot(), done);
        }
    }
}
#[test]
fn countered_omen_goes_to_graveyard_without_shuffle_or_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = spawn(&mut g, p, Card::from_name(NAMES[1]).unwrap(), Zone::Hand);
        stack(&mut g, id);
        let rng = g.state.rng;
        let library = g.state.players[p.index()].library.clone();
        let spell = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(spell, false));
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.rng, rng);
        assert_eq!(g.state.players[p.index()].library, library);
        assert!(g.state.adventures.is_none());
    }
}

#[test]
fn all_targets_illegal_fizzle_uses_live_max_power_or_absent_flying_and_never_shuffles() {
    for p in [Player::P0, Player::P1] {
        for name in [NAMES[3], NAMES[4]] {
            let mut g = setup(p);
            let id = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Hand);
            stack(&mut g, id);
            let target = g.state.stack.last().unwrap().targets[0].target;
            let Target::Card(victim) = target else {
                panic!("target")
            };
            if name == NAMES[3] {
                g.state.objects[victim as usize].growth.power = 2;
            } else {
                g.state.objects[victim as usize].gained_keywords = Keywords::FLYING;
            }
            let rng = g.state.rng;
            let library = g.state.players[p.index()].library.clone();
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.rng, rng);
            assert_eq!(g.state.players[p.index()].library, library);
            assert!(g.state.adventures.is_none());
        }
    }
}

#[test]
fn mandatory_discard_if_do_draw_has_no_decline_and_skips_draw_only_when_empty() {
    for p in [Player::P0, Player::P1] {
        for has_card in [false, true] {
            let mut g = setup(p);
            let id = spawn(&mut g, p, Card::from_name(NAMES[8]).unwrap(), Zone::Hand);
            let discard = has_card.then(|| spawn(&mut g, p, Card::Island, Zone::Hand));
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            stack(&mut g, id);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            if let Some(card) = discard {
                assert!(g.legal_actions().contains(&Action::Discard { card }));
                assert!(!g.legal_actions().contains(&Action::DeclineEffectPayment));
                assert!(g.observe(p.other()).effect_choice.is_none());
                act(&mut g, Action::Discard { card });
                assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                before + if has_card { 2 } else { 0 }
            );
            assert_eq!(g.state.objects[id as usize].zone, Zone::Library);
        }
    }
}
#[test]
fn late_source_generation_failure_restores_prior_search_shuffle_and_owned_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = spawn(&mut g, p, Card::from_name(NAMES[2]).unwrap(), Zone::Hand);
        g.state.objects[id as usize].generation = u64::MAX - 1;
        stack(&mut g, id);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        let selected = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSearch { .. }))
            .unwrap();
        act(&mut g, selected);
        let saved = g.snapshot();
        let rev = g.revision();
        for _ in 0..2 {
            assert!(matches!(
                g.apply(p, Action::FinishSearch),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), rev);
        }
    }
}
#[test]
fn purging_remove_all_counter_kinds_and_marang_two_other_targets_execute() {
    for p in [Player::P0, Player::P1] {
        for index in [5, 6] {
            let mut g = setup(p);
            let bears: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| o.zone == Zone::Battlefield)
                .map(|(i, _)| i as CardId)
                .collect();
            for &id in &bears {
                g.state.objects[id as usize].counters.plus_one = 2;
                g.state.objects[id as usize].counters.stun = 3;
                g.state.objects[id as usize].counters.loyalty = 4;
            }
            let id = spawn(
                &mut g,
                p,
                Card::from_name(NAMES[index]).unwrap(),
                Zone::Hand,
            );
            act(&mut g, Action::AnnounceCast { card: id });
            for _ in 0..100 {
                if g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.resolving.is_none()
                    && g.state.announcing.is_none()
                {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .min_by_key(|a| match a {
                        Action::ChooseTarget {
                            target: Target::Card(v),
                        } if bears.contains(v) => 0,
                        Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets => 1,
                        Action::Pass => 2,
                        Action::Concede => 1000,
                        _ => 100,
                    })
                    .unwrap();
                act(&mut g, a);
            }
            assert!(g.on_battlefield(id));
            if index == 5 {
                assert!(
                    bears
                        .iter()
                        .all(|&b| g.state.objects[b as usize].zone == Zone::Hand)
                );
            } else {
                assert!(bears.iter().any(|&b| g.counter_amounts(b).total() == 0));
            }
        }
    }
}
