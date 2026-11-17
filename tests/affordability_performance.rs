//! Semantic baseline traces and current-state staged payment regressions.
mod support;

use rustmage as _;
use rustmage_core::cards::Counters;
use rustmage_core::engine::*;
use rustmage_core::protocol::{Request, Session};
use support::*;

fn replay(trace: &str) -> Observation {
    let mut rows = trace
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap());
    let mut session =
        Session::new(serde_json::from_value(rows.next().unwrap()["arguments"].clone()).unwrap())
            .unwrap();
    let mut last = None;
    let mut expected = None;
    for (index, row) in rows.enumerate() {
        let request: Request = serde_json::from_value(row["request"].clone()).unwrap();
        let response = session.request(request);
        assert!(
            response.error.is_none(),
            "request {index}: {:?}",
            response.error
        );
        if let Some(boundary) = row.get("expected_boundary") {
            expected = Some(boundary.clone());
        }
        last = response.observation;
    }
    let observed = last.unwrap();
    if let Some(expected) = expected {
        let actual = serde_json::to_value(&observed).unwrap();
        for (key, value) in expected.as_object().unwrap() {
            assert_eq!(&actual[key], value, "baseline boundary differs in {key}");
        }
    }
    observed
}

#[test]
fn captured_enigma_observation_completes() {
    let observed = replay(include_str!("fixtures/affordability/enigma-semantic.jsonl"));
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
        "fixtures/affordability/groves-bounty-semantic.jsonl"
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

fn assert_no_payment_projection() {
    let work = Game::payment_planning_work();
    assert_eq!(work.projected_moves, 0);
    assert_eq!(work.projected_steps, 0);
    assert_eq!(work.continuation_plans, 0);
}

#[test]
fn craft_mana_shortfall_is_an_attempt_outcome_without_search() {
    for player in [Player::P0, Player::P1] {
        let (mut game, jewel) = craft_position(player, 5);
        game.state.players[player.index()].mana = [0; 6];
        Game::reset_payment_planning_work();
        act(&mut game, announce(jewel));
        act(&mut game, Action::BeginOrderedCosts);
        choose_clause(&mut game, CastCostClause::Mana);
        let revision = game.revision();
        act(&mut game, Action::SubmitAttempt);
        assert!(game.revision() > revision);
        assert_eq!(game.state.objects[jewel as usize].zone, Zone::Battlefield);
        assert!(game.state.announcing.is_none());
        assert_eq!(game.observe(player).last_attempt_outcome.unwrap().outcome, AttemptOutcome::Failed);
        assert!(game.legal_actions().contains(&announce(jewel)), "failure must permit retry");
        assert_no_payment_projection();
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
fn material_selection_does_not_project_linked_returns_or_attachments() {
    for player in [Player::P0, Player::P1] {
        let (mut game, jewel) = craft_position(player, 5);
        let material = put(&mut game, player, "Braided Net", Zone::Battlefield);
        let captive = put(&mut game, player, "Forest", Zone::Exile);
        game.state.exile_links.push(ExileLink {
            ability_origin: None,
            source: game.object_ref(material),
            exiled: game.object_ref(captive),
            return_zone: Zone::Battlefield,
        });
        act(&mut game, announce(jewel));
        act(&mut game, Action::BeginOrderedCosts);
        choose_clause(&mut game, CastCostClause::Additional);
        Game::reset_payment_planning_work();
        let choice = Action::ChooseAdditionalCost { object: game.object_ref(material) };
        assert!(game.legal_actions().contains(&choice));
        act(&mut game, choice);
        game.observe(player);
        assert_eq!(game.state.objects[material as usize].zone, Zone::Battlefield);
        assert_eq!(game.state.objects[captive as usize].zone, Zone::Exile);
        assert_no_payment_projection();
    }
}

#[test]
fn large_x_declarations_do_not_explore_life_paying_mana_sources() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        for _ in 0..3 {
            put(&mut game, player, "Starting Town", Zone::Battlefield);
        }
        let spell = put(&mut game, player, "Exsanguinate", Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];
        Game::reset_payment_planning_work();
        act(&mut game, Action::AnnounceCast { card: spell });
        assert!(game.x_choice().unwrap().contains(1 << 23));
        game.observe(player);
        act(&mut game, Action::ChooseX { value: 1 << 23 });
        act(&mut game, Action::EndManaWindow);
        act(&mut game, Action::SubmitAttempt);
        assert_eq!(game.observe(player).last_attempt_outcome.unwrap().outcome, AttemptOutcome::Failed);
        assert_no_payment_projection();
    }
}
