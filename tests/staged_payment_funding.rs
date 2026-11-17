//! Payment menus describe current choices; callers own future payment planning.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const CONVERTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const SACRIFICIAL_MANA: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 2, 0,
        ]))],
        ..CONVERTER.activated[0]
    }],
    ..CardRules::EMPTY
};

fn no_projection_work() {
    let work = Game::payment_planning_work();
    assert_eq!(work.projected_moves, 0);
    assert_eq!(work.projected_steps, 0);
    assert_eq!(work.continuation_plans, 0);
}

#[test]
fn a_concrete_child_payment_may_strand_its_parent_without_projection() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &CONVERTER)]);
    for player in [Player::P0, Player::P1] {
        let mut raw = scenario(player);
        let source = put(&mut raw, player, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut raw, player, "Divination", Zone::Hand);
        raw.state.players[player.index()].mana = [0, 1, 0, 0, 2, 0];
        Game::reset_payment_planning_work();
        act(&mut raw, Action::AnnounceCast { card: spell });
        let child = Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        };
        assert!(raw.legal_actions().contains(&child));
        raw.observe(player);
        raw.observe(player.other());
        let mut indexed = raw.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| action == &child)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut raw, child);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        let payment = Action::PayCosts {
            payment: [0, 1, 0, 0, 1, 0],
        };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| action == &payment)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut raw, payment);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[player.index()].mana, [0, 0, 0, 0, 2, 0]);
        assert_eq!(
            raw.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .source
                .unwrap()
                .id,
            spell
        );
        assert!(raw.mana_payment_domain().is_none());
        reject(
            &mut raw,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 1, 0],
            },
        );
        no_projection_work();
    }
}

#[test]
fn large_x_domain_and_invalid_concrete_payment_do_no_projection_work() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Exsanguinate", Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];
        Game::reset_payment_planning_work();
        act(&mut game, Action::AnnounceCast { card: spell });
        let choice = game.x_choice().expect("parameterized X declaration");
        assert!(choice.contains(1 << 20));
        game.observe(player);
        act(&mut game, Action::ChooseX { value: 1 << 20 });
        assert!(game.mana_payment_domain().is_none());
        reject(&mut game, Action::PayCosts { payment: [0; 6] });
        no_projection_work();
    }
}

#[test]
fn closing_the_mana_window_preserves_payment_but_hides_new_abilities() {
    let mut game = scenario(Player::P0);
    let forest = put(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "Divination", Zone::Hand);
    game.state.players[0].mana = [0, 1, 0, 0, 2, 0];
    Game::reset_payment_planning_work();
    act(&mut game, Action::AnnounceCast { card: spell });
    assert!(
        game.legal_actions()
            .contains(&Action::TapMana { card: forest })
    );
    act(&mut game, Action::EndManaWindow);
    assert!(
        !game
            .legal_actions()
            .contains(&Action::TapMana { card: forest })
    );
    reject(&mut game, Action::TapMana { card: forest });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 2, 0],
        },
    );
    assert!(game.state.announcing.is_none());
    no_projection_work();
}

#[test]
fn forage_without_current_materials_declares_a_route_then_fails_after_the_mana_window() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(
            &mut game,
            player,
            "Camellia, the Seedmiser",
            Zone::Battlefield,
        );
        let source_object = game.object_ref(source);
        let mana = game.state.players[player.index()].mana;
        let announce = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        Game::reset_payment_planning_work();
        act(&mut game, announce.clone());
        assert_eq!(game.state.pending, Pending::ForageMethod { player });
        assert_eq!(
            game.observe(player).forage.unwrap().routes,
            vec![ForageRoute::ExileThree, ForageRoute::Food]
        );
        let attempt = game.attempt_view(player).unwrap();
        assert_eq!(attempt.stage, AttemptStage::Declare);
        assert_eq!(attempt.locked_cost, None);
        assert!(!attempt.mana_window_closed);
        assert!(!attempt.can_end_mana_window);
        assert!(!attempt.can_submit);
        reject(&mut game, Action::EndManaWindow);
        reject(&mut game, Action::SubmitAttempt);
        act(
            &mut game,
            Action::ChooseForageRoute {
                route: ForageRoute::ExileThree,
            },
        );
        assert_eq!(game.state.pending, Pending::Forage { player });
        assert!(game.attempt_view(player).unwrap().can_end_mana_window);
        assert!(game.observe(player).forage.unwrap().candidates.is_empty());
        act(&mut game, Action::EndManaWindow);

        let mut indexed = game.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| *action == Action::SubmitAttempt)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, Action::SubmitAttempt);
        assert_eq!(game.snapshot(), indexed.snapshot());
        let outcome = game.last_attempt_outcome_view(player).unwrap();
        assert_eq!(outcome.outcome, AttemptOutcome::Failed);
        assert_eq!(
            outcome.reason,
            Some(AttemptFailureReason::UnsatisfiedMaterial)
        );
        assert_eq!(game.state.pending, Pending::Priority);
        assert!(game.state.announcing.is_none());
        assert!(game.state.stack.is_empty());
        assert_eq!(game.object_ref(source), source_object);
        assert_eq!(game.object_view(source).zone, Zone::Battlefield);
        assert!(!game.object_view(source).tapped);
        assert_eq!(game.state.players[player.index()].mana, mana);
        assert!(game.legal_actions().contains(&announce));

        // Recovery records the failed execution without hiding another attempt.
        act(&mut game, announce);
        assert!(game.attempt_view(player).unwrap().attempt_id > attempt.attempt_id);
        act(
            &mut game,
            Action::ChooseForageRoute {
                route: ForageRoute::Food,
            },
        );
        assert!(game.observe(player).forage.unwrap().candidates.is_empty());
        reject(
            &mut game,
            Action::ChooseForage {
                object: source_object,
            },
        );
        act(&mut game, Action::EndManaWindow);
        act(&mut game, Action::SubmitAttempt);
        let outcome = game.last_attempt_outcome_view(player).unwrap();
        assert_eq!(outcome.outcome, AttemptOutcome::Failed);
        assert_eq!(
            outcome.reason,
            Some(AttemptFailureReason::UnsatisfiedMaterial)
        );
        assert_eq!(game.state.pending, Pending::Priority);
        assert_eq!(game.object_ref(source), source_object);
        no_projection_work();
    }
}

#[test]
fn forage_can_use_a_graveyard_card_created_during_its_mana_window() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SACRIFICIAL_MANA)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(
            &mut game,
            player,
            "Camellia, the Seedmiser",
            Zone::Battlefield,
        );
        let squirrel = put(&mut game, player, "Barkform Harvester", Zone::Battlefield);
        let mana_source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let old_mana_source = game.object_ref(mana_source);
        let grave = [(); 2].map(|_| put(&mut game, player, "Forest", Zone::Graveyard));
        game.state.players[player.index()].mana = [0; 6];
        Game::reset_payment_planning_work();
        act(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(
            game.observe(player).forage.unwrap().routes,
            vec![ForageRoute::ExileThree, ForageRoute::Food]
        );
        act(
            &mut game,
            Action::ChooseForageRoute {
                route: ForageRoute::ExileThree,
            },
        );
        assert_eq!(game.state.pending, Pending::Forage { player });
        assert!(!game.attempt_view(player).unwrap().mana_window_closed);
        assert_eq!(game.observe(player).forage.unwrap().candidates.len(), 2);
        let make_material = Action::ActivateMana {
            card: mana_source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 2, 0],
        };
        assert!(game.legal_actions().contains(&Action::AnnounceMana {
            card: mana_source,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 2, 0],
        }));
        let mut closed = game.clone();
        act(&mut closed, Action::EndManaWindow);
        reject(&mut closed, make_material.clone());
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| action == &make_material)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, make_material.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.snapshot(), checkpoint);
        // Restore retains monotonic allocation history. Compare the replay
        // against a fork with that same history, not the first child's IDs.
        let mut replay = indexed.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| action == &make_material)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut replay, make_material);
        assert_eq!(replay.snapshot(), indexed.snapshot());
        assert_eq!(game.object_view(mana_source).zone, Zone::Graveyard);
        assert_ne!(game.object_ref(mana_source), old_mana_source);
        assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 2, 0]);
        assert_eq!(game.observe(player).forage.unwrap().candidates.len(), 3);
        act(&mut game, Action::EndManaWindow);
        reject(
            &mut game,
            Action::ChooseForage {
                object: old_mana_source,
            },
        );
        for card in grave.into_iter().chain([mana_source]) {
            let object = game.object_ref(card);
            act(&mut game, Action::ChooseForage { object });
        }
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 2, 0],
            },
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        for card in grave.into_iter().chain([mana_source]) {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
        }
        assert_eq!(
            game.last_attempt_outcome_view(player).unwrap().outcome,
            AttemptOutcome::Completed
        );
        settle(&mut game);
        assert_eq!(game.counter_count(squirrel, CounterKind::PlusOne), 1);
        no_projection_work();
    }
}

#[test]
fn completed_forage_with_missing_mana_fails_and_restores_the_exact_attempt() {
    for player in [Player::P0, Player::P1] {
        for route in [ForageRoute::ExileThree, ForageRoute::Food] {
            let mut game = scenario(player);
            let source = put(
                &mut game,
                player,
                "Camellia, the Seedmiser",
                Zone::Battlefield,
            );
            let squirrel = put(&mut game, player, "Barkform Harvester", Zone::Battlefield);
            let food = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
            let grave = [(); 3].map(|_| put(&mut game, player, "Forest", Zone::Graveyard));
            let source_object = game.object_ref(source);
            let food_object = game.object_ref(food);
            let grave_objects = grave.map(|card| game.object_ref(card));
            let materials = match route {
                ForageRoute::ExileThree => grave_objects.to_vec(),
                ForageRoute::Food => vec![food_object],
                _ => unreachable!(),
            };
            game.state.players[player.index()].mana = [0; 6];
            Game::reset_payment_planning_work();
            act(
                &mut game,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(&mut game, Action::ChooseForageRoute { route });
            for &object in &materials[..materials.len() - 1] {
                act(&mut game, Action::ChooseForage { object });
            }
            act(&mut game, Action::EndManaWindow);
            act(&mut game, Action::SubmitAttempt);
            assert_eq!(
                game.last_attempt_outcome_view(player).unwrap().outcome,
                AttemptOutcome::Incomplete
            );
            assert!(game.state.announcing.is_some());

            // Completing the separate forage selection must expose the actual
            // two-mana shortfall, rather than treating that selection as absent.
            act(
                &mut game,
                Action::ChooseForage {
                    object: *materials.last().unwrap(),
                },
            );
            assert_eq!(game.state.pending, Pending::PayCosts { player });
            let forage = game.observe(player).forage.unwrap();
            assert_eq!(forage.selected, materials);
            assert!(!forage.paid);
            assert!(game.attempt_view(player).unwrap().mana_window_closed);
            assert!(game.mana_payment_domain().is_none());
            reject(&mut game, Action::PayCosts { payment: [0; 6] });
            let checkpoint = game.snapshot();
            let mut indexed = game.clone();
            let index = indexed
                .legal_actions()
                .iter()
                .position(|action| *action == Action::SubmitAttempt)
                .unwrap();
            indexed
                .apply_index(player, indexed.revision(), index)
                .unwrap();
            act(&mut game, Action::SubmitAttempt);
            assert_eq!(game.snapshot(), indexed.snapshot());
            let outcome = game.last_attempt_outcome_view(player).unwrap();
            assert_eq!(outcome.outcome, AttemptOutcome::Failed);
            assert_eq!(outcome.reason, Some(AttemptFailureReason::UnsatisfiedMana));
            assert_eq!(game.state.pending, Pending::Priority);
            assert!(game.state.announcing.is_none());
            assert!(game.state.stack.is_empty());
            assert!(!game.legal_actions().contains(&Action::SubmitAttempt));
            assert_eq!(game.object_ref(source), source_object);
            assert_eq!(game.object_ref(food), food_object);
            assert_eq!(game.object_view(food).zone, Zone::Battlefield);
            for object in grave_objects {
                assert_eq!(game.object_ref(object.card), object);
                assert_eq!(game.object_view(object.card).zone, Zone::Graveyard);
            }
            assert_eq!(game.counter_count(squirrel, CounterKind::PlusOne), 0);
            assert_eq!(game.controlled_token_ids(player).count(), 0);
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);

            indexed.restore(&checkpoint).unwrap();
            act(&mut indexed, Action::SubmitAttempt);
            assert_eq!(game.snapshot(), indexed.snapshot());
            no_projection_work();
        }
    }
}

fn select_cost_clause(game: &mut Game, clause: CastCostClause) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        act(game, Action::BeginOrderedCosts);
    }
    let index = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && row.clause == clause)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause: index });
}

#[test]
fn teamwork_two_accepts_one_two_power_creature_and_pays_the_exact_cost() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let helper = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let target = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, player, "Repulsor Blast", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];
        Game::reset_payment_planning_work();
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseTeamwork { pay: true });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        act(&mut game, Action::EndManaWindow);
        reject(&mut game, Action::PayCosts { payment: [0; 6] });
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 3],
            },
        );
        select_cost_clause(&mut game, CastCostClause::Teamwork(2));
        reject(&mut game, Action::FinishTeamwork);
        let object = game.object_ref(helper);
        let teammate = Action::ChooseTeammate { object };
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| action == &teammate)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, teammate.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, teammate);
        assert_eq!(game.snapshot(), indexed.snapshot());
        let choice = game.teamwork_choice().unwrap();
        assert_eq!(choice.minimum, 2);
        assert_eq!(choice.selected, vec![object]);
        assert_eq!(choice.selected_power, Some(2));
        assert!(choice.can_finish);
        assert!(!game.object_view(helper).tapped);
        act(&mut game, Action::FinishTeamwork);
        assert!(game.object_view(helper).tapped);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 0, 0, 1, 0, 3]
        );
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().teamwork,
            Some(true)
        );
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.other().index()].life, 18);
        no_projection_work();
    }
}

#[test]
fn optional_material_costs_can_be_declared_before_materials_exist() {
    for player in [Player::P0, Player::P1] {
        for (name, intent, clause) in [
            (
                "Repulsor Blast",
                Action::ChooseTeamwork { pay: true },
                CastCostClause::Teamwork(2),
            ),
            (
                "Cinder Strike",
                Action::ChooseBlight { pay: true },
                CastCostClause::Blight {
                    amount: 1,
                    selected: None,
                },
            ),
            (
                "Caustic Exhale",
                Action::ChooseCastBehold { pay: true },
                CastCostClause::Behold("Dragon"),
            ),
        ] {
            let mut game = scenario(player);
            let target = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
            let target_object = game.object_ref(target);
            let spell = put(&mut game, player, name, Zone::Hand);
            let mana = game.state.players[player.index()].mana;
            Game::reset_payment_planning_work();
            act(&mut game, Action::AnnounceCast { card: spell });
            act(&mut game, intent);
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            assert!(game.attempt_view(player).unwrap().can_end_mana_window);
            act(&mut game, Action::EndManaWindow);
            if matches!(clause, CastCostClause::Teamwork(_)) {
                pay(&mut game);
            }
            select_cost_clause(&mut game, clause);
            let invalid = match clause {
                CastCostClause::Teamwork(_) => Action::ChooseTeammate {
                    object: target_object,
                },
                CastCostClause::Blight { .. } => Action::PayBlight {
                    object: target_object,
                },
                CastCostClause::Behold(_) => Action::ChooseBeholdCost {
                    object: target_object,
                },
                _ => unreachable!(),
            };
            reject(&mut game, invalid);
            let checkpoint = game.snapshot();
            let mut indexed = game.clone();
            let index = indexed
                .legal_actions()
                .iter()
                .position(|action| *action == Action::SubmitAttempt)
                .unwrap();
            indexed
                .apply_index(player, indexed.revision(), index)
                .unwrap();
            act(&mut game, Action::SubmitAttempt);
            assert_eq!(game.snapshot(), indexed.snapshot());
            indexed.restore(&checkpoint).unwrap();
            act(&mut indexed, Action::SubmitAttempt);
            assert_eq!(game.snapshot(), indexed.snapshot());
            let outcome = game.last_attempt_outcome_view(player).unwrap();
            assert_eq!(outcome.outcome, AttemptOutcome::Failed);
            assert_eq!(
                outcome.reason,
                Some(AttemptFailureReason::UnsatisfiedMaterial)
            );
            assert_eq!(game.object_view(spell).zone, Zone::Hand);
            assert_eq!(game.object_ref(target), target_object);
            assert!(!game.object_view(target).tapped);
            assert_eq!(game.counter_count(target, CounterKind::MinusOne), 0);
            assert_eq!(game.state.players[player.index()].mana, mana);
            assert_eq!(game.state.players[player.other().index()].life, 20);
            no_projection_work();
        }
    }
}

#[test]
fn bargain_declares_before_material_selection_and_missing_materials_fail() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let target = put(
            &mut game,
            player.other(),
            "Gigantosaurus",
            Zone::Battlefield,
        );
        let target_object = game.object_ref(target);
        let spell = put(&mut game, player, "Candy Grapple", Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];
        Game::reset_payment_planning_work();
        act(&mut game, Action::AnnounceCast { card: spell });
        assert!(
            game.legal_actions()
                .contains(&Action::ChooseBargain { sacrifice: None })
        );
        let mut declined = game.clone();
        act(&mut declined, Action::DeclareBargain { pay: false });
        assert_eq!(
            declined
                .state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .cast
                .unwrap()
                .bargain,
            Some(false)
        );
        let declaration = Action::DeclareBargain { pay: true };
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| action == &declaration)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, declaration.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, declaration);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.state.announcing.as_ref().unwrap().additional, None);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        assert_eq!(game.state.pending, Pending::AdditionalCost { player });
        assert!(game.attempt_view(player).unwrap().can_end_mana_window);
        let mut failed = game.clone();
        act(&mut failed, Action::EndManaWindow);
        act(&mut failed, Action::SubmitAttempt);
        let outcome = failed.last_attempt_outcome_view(player).unwrap();
        assert_eq!(outcome.outcome, AttemptOutcome::Failed);
        assert_eq!(
            outcome.reason,
            Some(AttemptFailureReason::UnsatisfiedMaterial)
        );
        assert_eq!(failed.object_view(spell).zone, Zone::Hand);
        assert_eq!(failed.object_ref(target), target_object);
        assert!(!failed.object_view(creature).tapped);
        assert_eq!(failed.state.players[player.index()].mana, [0; 6]);

        let offering = put(&mut failed, player, "Candy Trail", Zone::Battlefield);
        let object = failed.object_ref(offering);
        failed.state.players[player.index()].mana = [0, 0, 2, 0, 0, 0];
        act(&mut failed, Action::AnnounceCast { card: spell });
        for shortcut in [false, true] {
            let mut retry = failed.clone();
            act(
                &mut retry,
                if shortcut {
                    Action::ChooseBargain {
                        sacrifice: Some(object),
                    }
                } else {
                    Action::DeclareBargain { pay: true }
                },
            );
            assert_eq!(
                retry.state.announcing.as_ref().unwrap().additional,
                shortcut.then_some(object)
            );
            act(
                &mut retry,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            act(&mut retry, Action::EndManaWindow);
            if !shortcut {
                act(&mut retry, Action::ChooseAdditionalCost { object });
            }
            assert!(
                retry.on_battlefield(offering),
                "bargain material stays provisional until payment"
            );
            assert_eq!(
                retry.state.announcing.as_ref().unwrap().additional,
                Some(object)
            );
            reject(&mut retry, Action::PayCosts { payment: [0; 6] });
            act(
                &mut retry,
                Action::PayCosts {
                    payment: [0, 0, 2, 0, 0, 0],
                },
            );
            assert_eq!(retry.object_view(offering).zone, Zone::Graveyard);
            assert_eq!(retry.state.players[player.index()].mana, [0; 6]);
            assert_eq!(
                retry.state.stack.last().unwrap().cast.unwrap().bargain,
                Some(true)
            );
            settle(&mut retry);
            assert_eq!(stats(&retry, target), (5, 5));
        }
        no_projection_work();
    }
}

const TAP_AND_WATERBEND: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        program: &[
            Instruction::WaterbendCost(1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
        ..CONVERTER.activated[0]
    }],
    ..CardRules::EMPTY
};

#[test]
fn concrete_waterbend_cannot_pay_the_same_source_tap_twice() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TAP_AND_WATERBEND)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let helper = put(&mut game, Player::P0, "Goblin Piker", Zone::Battlefield);
    game.state.players[0].mana = [0; 6];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(&mut game, Action::BeginWaterbend);
    let mut invalid = game.clone();
    // Supply a complete conflicting clause, independently of prefix filtering.
    let reference = invalid.object_ref(source);
    let frame = invalid
        .state
        .announcing
        .as_mut()
        .unwrap()
        .convoke
        .as_mut()
        .unwrap();
    frame.substituted.push(reference);
    frame.remaining.generic = 0;
    assert!(invalid.mana_payment_domain().is_none());
    assert!(
        !invalid
            .legal_actions()
            .contains(&Action::PayCosts { payment: [0; 6] })
    );
    reject(&mut invalid, Action::PayCosts { payment: [0; 6] });
    assert!(!invalid.object_view(source).tapped);
    let helper = game.object_ref(helper);
    act(&mut game, Action::ChooseWaterbender { object: helper });
    act(&mut game, Action::PayCosts { payment: [0; 6] });
    assert!(game.object_view(source).tapped);
    assert!(game.object_view(helper.card).tapped);
    assert!(game.state.announcing.is_none());
}

const SHARED_COUNTERS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                remove: Some((Some(CounterKind::PlusOne), 1)),
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::RemoveCreatureCounter(CounterKind::PlusOne),
                    filter: SacrificeFilter {
                        kinds: 1,
                        source_only: false,
                        other: false,
                        nonland: false,
                        tokens: None,
                        subtype: None,
                        keywords: Keywords::NONE,
                        artifact_enchantment_or_token: false,
                    },
                }),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            ..CONVERTER.activated[0]
        },
        ActivatedAbility {
            cost: ActivationCost {
                remove: Some((Some(CounterKind::PlusOne), 1)),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            ..CONVERTER.activated[0]
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn a_nested_payment_cannot_make_one_counter_pay_two_parent_costs() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SHARED_COUNTERS)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[source as usize].counters.plus_one = 2;
    game.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::of(CounterKind::PlusOne, 1),
        },
    );
    let object = game.object_ref(source);
    act(
        &mut game,
        Action::ChooseCreatureCounterCost {
            object,
            counter: CounterKind::PlusOne,
        },
    );
    let mut valid = game.clone();
    act(
        &mut valid,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(valid.state.objects[source as usize].counters.plus_one, 0);
    assert!(valid.state.announcing.is_none());

    act(
        &mut game,
        Action::AnnounceMana {
            card: source,
            ability: 1,
            counter_payment: Counters::of(CounterKind::PlusOne, 1),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    act(&mut game, Action::PayCosts { payment: [0; 6] });
    assert_eq!(game.state.objects[source as usize].counters.plus_one, 1);
    assert!(game.mana_payment_domain().is_none());
    assert!(!game.legal_actions().contains(&Action::PayCosts {
        payment: [0, 0, 0, 0, 0, 1]
    }));
    reject(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
}

const LIFE_THEN_SACRIFICE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    life: Some(AmountValue::Fixed(2)),
                    objects: Some(ResolutionObjectCost {
                        cost: AdditionalCost {
                            kind: EffectChoiceKind::Sacrifice,
                            filter: SacrificeFilter {
                                kinds: 1,
                                source_only: false,
                                other: false,
                                nonland: false,
                                tokens: None,
                                subtype: None,
                                keywords: Keywords::NONE,
                                artifact_enchantment_or_token: false,
                            },
                        },
                        count: 1,
                    }),
                    ..ResolutionCost::FREE
                }],
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    ),
    ..CardRules::EMPTY
};

const IMPLICIT_SOURCE_EXILE: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            exile_self: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileSourceAndNamed {
                    name: "Divination",
                    amount: 1,
                },
                filter: SacrificeFilter {
                    kinds: 0,
                    source_only: false,
                    other: false,
                    nonland: false,
                    tokens: None,
                    subtype: None,
                    keywords: Keywords::NONE,
                    artifact_enchantment_or_token: false,
                },
            }),
            ..ActivationCost::ZERO
        },
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        origin: ActivationOrigin::Graveyard,
        ..CONVERTER.activated[0]
    }],
    ..CardRules::EMPTY
};

#[test]
fn implicit_source_exile_cannot_pay_a_second_source_departure() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &IMPLICIT_SOURCE_EXILE)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Divination", Zone::Graveyard);
    let named = put(&mut game, Player::P0, "Divination", Zone::Graveyard);
    game.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = game.object_ref(named);
    act(&mut game, Action::ChooseAdditionalCost { object });
    assert!(game.mana_payment_domain().is_none());
    reject(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(named).zone, Zone::Graveyard);
}

#[test]
fn fixed_resolution_life_replacement_matches_execution_without_future_search() {
    use rustmage_core::engine::ability_world::CharacteristicWorld;

    struct ClearPrevention;
    impl Drop for ClearPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let _clear = ClearPrevention;
    let _rules = fixture_rules::activate(&[(Card::Divination, &LIFE_THEN_SACRIFICE)]);
    for prevented in [false, true] {
        let mut game = scenario(Player::P0);
        let ashiok = put(
            &mut game,
            Player::P0,
            "Ashiok, Wicked Manipulator",
            Zone::Battlefield,
        );
        game.state.objects[ashiok as usize].counters.loyalty = 5;
        let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut game, Player::P0, "Divination", Zone::Hand);
        let top = game.state.players[0]
            .library
            .iter()
            .rev()
            .take(2)
            .map(|&id| game.object_ref(id))
            .collect::<Vec<_>>();
        zone_replacements::TEST_PREVENT_ZONE_MOVE
            .with(|rule| rule.set(prevented.then_some(top[0])));
        Game::reset_payment_planning_work();
        let world = game.project_resolution_life(Player::P0, 2);
        assert_eq!(world.life(Player::P0), 20);
        assert_eq!(
            world.object(top[0].card).zone,
            if prevented {
                Zone::Library
            } else {
                Zone::Exile
            }
        );
        assert_eq!(world.object(top[1].card).zone, Zone::Exile);
        cast(&mut game, spell, None);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        let object = game.object_ref(creature);
        act(&mut game, Action::ChooseAdditionalCost { object });
        let payment = Action::PayResolutionMana { payment: [0; 6] };
        let mut indexed = game.clone();
        let before = indexed.snapshot();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| a == &payment)
            .unwrap();
        indexed
            .apply_index(Player::P0, indexed.revision(), index)
            .unwrap();
        act(&mut game, payment);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(game.state.players[0].life, 23);
        assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
        assert_eq!(
            game.object_view(top[0].card).zone,
            if prevented {
                Zone::Library
            } else {
                Zone::Exile
            }
        );
        assert_eq!(game.object_view(top[1].card).zone, Zone::Exile);
        indexed.restore(&before).unwrap();
        assert_eq!(indexed.object_view(top[1].card).zone, Zone::Library);
        no_projection_work();
    }
}

const HAND_MANA: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            exile_self: true,
            ..ActivationCost::ZERO
        },
        origin: ActivationOrigin::Hand,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        ..CONVERTER.activated[0]
    }],
    ..CardRules::EMPTY
};

#[test]
fn a_held_face_down_draw_cannot_reveal_its_hand_mana_ability() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &HAND_MANA)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Bear Cub", Zone::Hand);
        let drawn = put(&mut game, player, "Grizzly Bears", Zone::Library);
        act(&mut game, Action::AnnounceCast { card: spell });
        assert!(game.draw_one_unreplaced(player));
        assert!(game.attempt_draw_is_face_down(game.object_ref(drawn)));
        let announce = Action::AnnounceMana {
            card: drawn,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        };
        let direct = Action::ActivateMana {
            card: drawn,
            ability: 0,
            counter_payment: Counters::default(),
            payment: [0; 6],
            production: [0, 0, 0, 0, 1, 0],
        };
        for action in [&announce, &direct] {
            assert!(!game.legal_actions().contains(action));
            reject(&mut game, action.clone());
        }
        let mut indexed = game.clone();
        let pay = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        let at = indexed
            .legal_actions()
            .iter()
            .position(|a| a == &pay)
            .unwrap();
        indexed.apply_index(player, indexed.revision(), at).unwrap();
        act(&mut game, pay);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert!(!game.attempt_draw_is_face_down(game.object_ref(drawn)));
        assert!(game.legal_actions().contains(&announce));
    }
}

#[test]
fn combat_prefixes_wait_for_complete_declarations_without_suffix_search() {
    const COMPANY: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CantAttackOrBlockAlone],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &COMPANY)]);
    for taxed in [false, true] {
        let mut game = scenario(Player::P0);
        let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
        if taxed {
            put(
                &mut game,
                Player::P1,
                "Archangel of Tithes",
                Zone::Battlefield,
            );
        }
        game.state.players[0].mana = [0; 6];
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        let action = Action::Attack { card: source };
        let mut indexed = game.clone();
        let at = indexed
            .legal_actions()
            .iter()
            .position(|a| a == &action)
            .unwrap();
        indexed
            .apply_index(Player::P0, indexed.revision(), at)
            .unwrap();
        act(&mut game, action);
        assert_eq!(indexed.snapshot(), game.snapshot());
        reject(&mut game, Action::FinishAttackers);
        act(&mut game, Action::CancelAttackDeclaration);
        assert!(game.state.selected_attackers.is_empty());
        let companion = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
        act(&mut game, Action::Attack { card: source });
        act(&mut game, Action::Attack { card: companion });
        act(&mut game, Action::FinishAttackers);
        if taxed {
            assert_eq!(game.observe(Player::P0).attack_payment.unwrap().generic, 2);
            assert_eq!(game.state.players[0].mana, [0; 6]);
            reject(&mut game, Action::PayAttackCost { payment: [0; 6] });
            act(&mut game, Action::AbortAttackPayment);
        }
    }
}

#[test]
fn an_unpaid_counter_clause_can_enter_ordered_costs_after_the_mana_window() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Sunstar Chaplain", Zone::Battlefield);
        let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let original = game.object_ref(source);
        game.state.players[player.index()].mana = [0; 6];
        Game::reset_payment_planning_work();
        act(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        act(&mut game, Action::EndManaWindow);
        assert!(game.legal_actions().contains(&Action::BeginOrderedCosts));
        act(&mut game, Action::BeginOrderedCosts);
        let clause = game
            .observe(player)
            .cast_costs
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Additional && !row.paid)
            .unwrap() as u16;
        act(&mut game, Action::ChooseCastCost { clause });
        act(&mut game, Action::SubmitAttempt);
        let outcome = game.last_attempt_outcome_view(player).unwrap();
        assert_eq!(outcome.outcome, AttemptOutcome::Failed);
        assert_eq!(
            outcome.reason,
            Some(AttemptFailureReason::UnsatisfiedMaterial)
        );
        assert_eq!(game.object_ref(source), original);
        assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        no_projection_work();
    }
}
