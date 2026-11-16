//! Regression traces from issue #684 and exact fixed-world pruning witnesses.
mod support;

use rustmage as _;
use rustmage_core::cards::Counters;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use rustmage_core::protocol::{Request, Session};
use std::collections::HashSet;
use support::*;

fn replay(trace: &str) -> Observation {
    let mut rows = trace
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap());
    let mut session =
        Session::new(serde_json::from_value(rows.next().unwrap()["arguments"].clone()).unwrap())
            .unwrap();
    let mut last = None;
    for (index, row) in rows.enumerate() {
        let request: Request = serde_json::from_value(row["request"].clone()).unwrap();
        let response = session.request(request);
        assert!(
            response.error.is_none(),
            "request {index}: {:?}",
            response.error
        );
        last = response.observation;
    }
    last.unwrap()
}

#[test]
fn captured_enigma_observation_completes() {
    let observed = replay(include_str!("fixtures/affordability/enigma-requests.jsonl"));
    assert_eq!(
        (observed.turn, observed.step, observed.active),
        (12, Step::Main1, Player::P0)
    );
    assert!(
        observed
            .objects
            .iter()
            .any(|o| o.name == "The Enigma Jewel" && o.zone == Zone::Battlefield && !o.tapped)
    );
    assert_eq!(
        observed
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.controller == Player::P0
                && o.card.effect() == rustmage_core::cards::Effect::Land)
            .count(),
        5
    );
}

#[test]
fn captured_groves_bounty_x_probes_complete() {
    replay(include_str!(
        "fixtures/affordability/groves-bounty-requests.jsonl"
    ));
}

fn announce(jewel: CardId) -> Action {
    Action::AnnounceActivate {
        card: jewel,
        ability: 1,
        counter_payment: Counters::default(),
    }
}

fn craft_position(p: Player, islands: usize) -> (Game, CardId) {
    let mut game = scenario(p);
    let jewel = put(&mut game, p, "The Enigma Jewel", Zone::Battlefield);
    for _ in 0..islands {
        put(&mut game, p, "Island", Zone::Battlefield);
    }
    for name in [
        "Braided Net",
        "Cryogen Relic",
        "Repurposing Bay",
        "The Mind Stone",
    ] {
        put(&mut game, p, name, Zone::Graveyard);
    }
    let snare = put(&mut game, p, "Perilous Snare", Zone::Battlefield);
    let captive = put(&mut game, p.other(), "Island", Zone::Exile);
    game.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: game.object_ref(snare),
        exiled: game.object_ref(captive),
        return_zone: Zone::Battlefield,
    });
    (game, jewel)
}

#[test]
fn craft_mana_shortfall_is_pruned_before_any_search_expansion() {
    for p in [Player::P0, Player::P1] {
        let (mut game, jewel) = craft_position(p, 5);
        let mut rejected = game.clone();
        rejected.state.players[p.index()].mana = [0; 6];
        reject(&mut rejected, announce(jewel));
        // Enter the actual ordered cost context, then expose its mana shortfall.
        act(&mut game, announce(jewel));
        act(&mut game, Action::BeginOrderedCosts);
        game.state.players[p.index()].mana = [0; 6];
        let a = game.state.announcing.as_ref().unwrap();
        let goal = game.funding_goal(a);
        let mut world = FundingState::from_game(&game, &goal);
        world.set_cost_context(Some(a));
        let mut visited = HashSet::new();
        assert!(world.fixed_mana_cannot_pay(&goal));
        assert!(!world.reachable(&goal, &mut visited));
        assert!(
            visited.is_empty(),
            "negative craft requires no expanded states"
        );
        let before = game.snapshot();
        game.legal_actions();
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn jewel_restricted_mana_keeps_a_payable_craft_and_owned_branches() {
    for p in [Player::P0, Player::P1] {
        let (mut game, jewel) = craft_position(p, 7);
        game.state.players[p.index()].mana = [0; 6];
        let action = announce(jewel);
        let saved = game.snapshot();
        let actions = game.legal_actions();
        let index = actions.iter().position(|a| *a == action).unwrap();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        drop(game);
        act(&mut raw, action);
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for branch in [&mut raw, &mut indexed] {
            act(
                branch,
                Action::ActivateMana {
                    card: jewel,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 0, 0, 0, 0, 2],
                },
            );
            let islands = branch
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.zone == Zone::Battlefield && o.card == rustmage_core::cards::Card::Island
                })
                .map(|(id, _)| id as CardId)
                .collect::<Vec<_>>();
            for card in islands {
                act(branch, Action::TapMana { card });
            }
            assert_eq!(branch.state.players[p.index()].mana, [0, 7, 0, 0, 0, 2]);
            act(branch, Action::BeginOrderedCosts);
            choose_clause(branch, CastCostClause::Mana);
            pay(branch);
            choose_clause(branch, CastCostClause::Additional);
            let materials = branch
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| o.zone == Zone::Graveyard)
                .map(|(id, _)| id as CardId)
                .collect::<Vec<_>>();
            assert_eq!(materials.len(), 4);
            for card in materials {
                act(
                    branch,
                    Action::ChooseAdditionalCost {
                        object: branch.object_ref(card),
                    },
                );
            }
            act(branch, Action::FinishCraftMaterials);
            choose_clause(branch, CastCostClause::CraftSource);
            settle(branch);
            assert_eq!(
                branch.object_view(jewel).card.name(),
                "Locus of Enlightenment"
            );
            assert_eq!(
                branch.crafted_exiled_cards(branch.object_ref(jewel)).len(),
                4
            );
            assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        raw.restore(&saved).unwrap();
        assert_eq!(raw.snapshot(), saved);
        assert_eq!(raw.legal_actions(), actions);
    }
}

fn choose_clause(game: &mut Game, clause: CastCostClause) {
    let index = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == clause && !r.paid)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause: index });
}

#[test]
fn craft_retains_linked_source_and_attachment_fallbacks() {
    for p in [Player::P0, Player::P1] {
        for attached in [false, true] {
            let (mut game, jewel) = craft_position(p, 5);
            let material = put(&mut game, p, "Braided Net", Zone::Battlefield);
            if attached {
                let equipment = put(&mut game, p, "Stalactite Dagger", Zone::Battlefield);
                let creature = put(&mut game, p, "Llanowar Elves", Zone::Battlefield);
                assert!(
                    game.attach_equipment(game.object_ref(equipment), game.object_ref(creature))
                );
            } else {
                let captive = put(&mut game, p, "Forest", Zone::Exile);
                game.state.exile_links.push(ExileLink {
                    ability_origin: None,
                    source: game.object_ref(material),
                    exiled: game.object_ref(captive),
                    return_zone: Zone::Battlefield,
                });
            }
            act(&mut game, announce(jewel));
            act(&mut game, Action::BeginOrderedCosts);
            game.state.players[p.index()].mana = [0; 6];
            let a = game.state.announcing.as_ref().unwrap();
            let goal = game.funding_goal(a);
            let mut world = FundingState::from_game(&game, &goal);
            world.set_cost_context(Some(a));
            if !attached {
                assert!(world.fixed_mana_link_can_return(&goal, game.object_ref(material)));
            }
            assert!(!world.fixed_mana_cannot_pay(&goal));
        }
    }
}

#[test]
fn life_paying_tap_sources_bound_repeated_negative_x_probes() {
    for p in [Player::P0, Player::P1] {
        let mut game = scenario(p);
        for _ in 0..3 {
            put(&mut game, p, "Starting Town", Zone::Battlefield);
        }
        for _ in 0..2 {
            let land = put(&mut game, p, "Multiversal Passage", Zone::Battlefield);
            game.set_entry_value(
                game.object_ref(land),
                rustmage_core::cards::EntryChoiceValue::BasicLandType(
                    rustmage_core::cards::BasicLandType::Forest,
                ),
            );
        }
        put(&mut game, p.other(), "Icetill Explorer", Zone::Battlefield);
        put(&mut game, p, "Elusive Otter", Zone::Battlefield);
        let spell = put(&mut game, p, "Traumatic Critique", Zone::Hand);
        game.state.players[p.index()].mana = [0; 6];
        for generic in [4, 8, 1024, 1 << 23] {
            let goal = FundingGoal {
                actor: p,
                required: [0, 1, 0, 1, 0, 0],
                generic,
                reserve: None,
                source: None,
            };
            let mut world = FundingState::from_game(&game, &goal);
            world.set_purpose(game.spell_mana_purpose_for_card(
                p,
                spell,
                game.state.objects[spell as usize].card,
            ));
            let mut visited = HashSet::new();
            assert!(!world.reachable(&goal, &mut visited));
            assert!(visited.is_empty());
        }
    }
}

#[test]
fn captured_groves_bounty_negative_probe_expands_no_states() {
    let mut rows = include_str!("fixtures/affordability/groves-bounty-requests.jsonl")
        .lines()
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap());
    let mut game =
        Game::new(serde_json::from_value(rows.next().unwrap()["arguments"].clone()).unwrap())
            .unwrap();
    for row in rows.take(725) {
        match serde_json::from_value(row["request"].clone()).unwrap() {
            Request::Step {
                actor,
                revision,
                index,
                viewer,
            } => {
                game.apply_index(actor, revision, index).unwrap();
                game.observe(viewer);
            }
            Request::Act {
                actor,
                action,
                viewer,
                ..
            } => {
                game.apply(actor, action).unwrap();
                game.observe(viewer);
            }
            Request::Observe { viewer } => {
                game.observe(viewer);
            }
            _ => panic!("unexpected request"),
        }
    }
    let goal = FundingGoal {
        actor: Player::P1,
        required: [0, 0, 0, 0, 1, 0],
        generic: 5,
        reserve: None,
        source: None,
    };
    let spell = game
        .state
        .objects
        .iter()
        .position(|o| {
            o.zone == Zone::Hand && o.owner == Player::P1 && o.card.name() == "Elusive Otter"
        })
        .unwrap() as CardId;
    let mut world = FundingState::from_game(&game, &goal);
    world.set_purpose(game.spell_mana_purpose_for_card(
        Player::P1,
        spell,
        rustmage_core::cards::Card::from_name("Grove's Bounty").unwrap(),
    ));
    assert!(world.fixed_mana_cannot_pay(&goal));
    let mut visited = HashSet::new();
    assert!(!world.reachable(&goal, &mut visited));
    assert!(visited.is_empty());
}
