//! Entering ordered costs retains the complete initial mana/source payment.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ZERO: Mana = [0; 6];
const EXTRA: Mana = [0, 0, 0, 0, 0, 1];
const TWO: Mana = [0, 0, 0, 0, 0, 2];
const COUNTER_SOURCE: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            life: 2,
            remove: Some((Some(CounterKind::PlusOne), 1)),
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::RemoveCreatureCounter(CounterKind::PlusOne),
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::Tap],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn same_step(game: &mut Game, action: Action) {
    let actor = game.actor().unwrap();
    let actions = game.legal_actions();
    let index = actions
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap_or_else(|| {
            panic!(
                "missing {action:?} in {:?}: {actions:?}",
                game.state.pending
            )
        });
    let revision = game.revision();
    let mut indexed = game.clone();
    indexed.apply_index(actor, revision, index).unwrap();
    act(game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
}
fn enter_ordered(game: &mut Game) {
    if matches!(game.state.pending, Pending::AdditionalCost { .. }) {
        same_step(game, Action::BeginOrderedCosts);
    }
    same_step(game, Action::EndManaWindow);
    same_step(game, Action::BeginOrderedCosts);
}
fn clause(game: &mut Game, clause: CastCostClause) {
    let rows = &game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses;
    let index = rows
        .iter()
        .position(|row| row.clause == clause && !row.paid)
        .unwrap();
    same_step(
        game,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}
fn assert_mana_unpaid(game: &Game) {
    let costs = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap();
    assert!(!costs.mana_paid);
    let rows: Vec<_> = costs
        .clauses
        .iter()
        .filter(|row| row.clause == CastCostClause::Mana)
        .collect();
    assert_eq!(rows.len(), 1);
    assert!(!rows[0].paid);
    assert!(game.state.stack.is_empty());
}
fn fail_unpaid(game: &mut Game, player: Player) {
    assert_mana_unpaid(game);
    clause(game, CastCostClause::Mana);
    assert!(game.mana_payment_domain().is_none());
    reject(game, Action::PayCosts { payment: ZERO });
    same_step(game, Action::SubmitAttempt);
    let outcome = game.last_attempt_outcome_view(player).unwrap();
    assert_eq!(outcome.outcome, AttemptOutcome::Failed);
    assert_eq!(outcome.reason, Some(AttemptFailureReason::UnsatisfiedMana));
    assert_eq!(game.state.pending, Pending::Priority);
    assert!(game.state.announcing.is_none());
    assert!(game.state.stack.is_empty());
}
fn add_mana(payment: Mana) -> Mana {
    std::array::from_fn(|index| payment[index] + EXTRA[index])
}
fn assert_no_projection() {
    let work = Game::payment_planning_work();
    assert_eq!(work.projected_moves, 0);
    assert_eq!(work.projected_steps, 0);
    assert_eq!(work.continuation_plans, 0);
}

#[test]
fn teamwork_ordered_costs_cannot_replace_the_initial_mana_payment() {
    for player in [Player::P0, Player::P1] {
        for funded in [false, true] {
            for mana_first in [false, true].into_iter().filter(|first| funded || !first) {
                let mut game = scenario(player);
                let spell = put(&mut game, player, "Team Tactics", Zone::Hand);
                let helper = put(&mut game, player, "Bear Cub", Zone::Battlefield);
                let helper = game.object_ref(helper);
                let payment = [0, 0, 0, 1, 0, 1];
                game.state.players[player.index()].mana =
                    if funded { add_mana(payment) } else { ZERO };
                Game::reset_payment_planning_work();
                act(&mut game, Action::AnnounceCast { card: spell });
                act(&mut game, Action::ChooseTeamwork { pay: true });
                act(
                    &mut game,
                    Action::ChooseTarget {
                        target: Target::Card(helper.card),
                    },
                );
                assert_mana_unpaid(&game);
                enter_ordered(&mut game);
                if mana_first {
                    clause(&mut game, CastCostClause::Mana);
                    same_step(&mut game, Action::PayCosts { payment });
                    assert_eq!(game.state.players[player.index()].mana, EXTRA);
                    reject(&mut game, Action::PayCosts { payment });
                }
                clause(&mut game, CastCostClause::Teamwork(1));
                same_step(&mut game, Action::ChooseTeammate { object: helper });
                same_step(&mut game, Action::FinishTeamwork);
                assert!(game.object_view(helper.card).tapped);
                if !funded {
                    fail_unpaid(&mut game, player);
                    assert_eq!(game.object_view(spell).zone, Zone::Hand);
                    assert_eq!(game.object_ref(helper.card), helper);
                    assert!(!game.object_view(helper.card).tapped);
                } else {
                    if !mana_first {
                        assert_mana_unpaid(&game);
                        clause(&mut game, CastCostClause::Mana);
                        same_step(&mut game, Action::PayCosts { payment });
                    }
                    assert_eq!(game.state.players[player.index()].mana, EXTRA);
                    let item = game
                        .state
                        .stack
                        .iter()
                        .find(|item| item.source.is_some_and(|source| source.id == spell))
                        .unwrap();
                    assert_eq!(item.cast.unwrap().payment, payment);
                    assert_eq!(item.cast.unwrap().teamwork, Some(true));
                    assert!(game.state.announcing.is_none());
                }
                assert_no_projection();
            }
        }
    }
}

#[test]
fn casualty_ordered_costs_cannot_replace_the_initial_mana_payment() {
    for player in [Player::P0, Player::P1] {
        for funded in [false, true] {
            for mana_first in [false, true].into_iter().filter(|first| funded || !first) {
                let mut game = scenario(player);
                put(
                    &mut game,
                    player,
                    "Silverquill, the Disputant",
                    Zone::Battlefield,
                );
                let spell = put(&mut game, player, "Divination", Zone::Hand);
                let offering = put(&mut game, player, "Bear Cub", Zone::Battlefield);
                let offering = game.object_ref(offering);
                let payment = [0, 1, 0, 0, 0, 2];
                game.state.players[player.index()].mana =
                    if funded { add_mana(payment) } else { ZERO };
                Game::reset_payment_planning_work();
                act(&mut game, Action::AnnounceCast { card: spell });
                act(&mut game, Action::ChooseCasualty { pay: true });
                assert_mana_unpaid(&game);
                enter_ordered(&mut game);
                if mana_first {
                    clause(&mut game, CastCostClause::Mana);
                    same_step(&mut game, Action::PayCosts { payment });
                    assert_eq!(game.state.players[player.index()].mana, EXTRA);
                    reject(&mut game, Action::PayCosts { payment });
                }
                clause(&mut game, CastCostClause::Casualty(0));
                same_step(&mut game, Action::PayCasualty { object: offering });
                assert_eq!(game.object_view(offering.card).zone, Zone::Graveyard);
                if !funded {
                    fail_unpaid(&mut game, player);
                    assert_eq!(game.object_view(spell).zone, Zone::Hand);
                    assert_eq!(game.object_ref(offering.card), offering);
                    assert_eq!(game.object_view(offering.card).zone, Zone::Battlefield);
                } else {
                    if !mana_first {
                        assert_mana_unpaid(&game);
                        clause(&mut game, CastCostClause::Mana);
                        same_step(&mut game, Action::PayCosts { payment });
                    }
                    assert_eq!(game.state.players[player.index()].mana, EXTRA);
                    let item = game
                        .state
                        .stack
                        .iter()
                        .find(|item| {
                            item.source.is_some_and(|source| source.id == spell)
                                && item.kind == StackKind::Spell
                        })
                        .unwrap();
                    assert_eq!(item.cast.unwrap().payment, payment);
                    assert!(game.state.announcing.is_none());
                    let observation = game.observe(player);
                    assert!(
                        observation
                            .stack
                            .iter()
                            .chain(&observation.waiting_triggers)
                            .any(|item| item.kind == StackKind::CasualtyCopy)
                    );
                }
                assert_no_projection();
            }
        }
    }
}

#[test]
fn ordered_counter_costs_commit_activation_mana_tap_life_and_source_counters_once() {
    let definition = Card::from_name("Sunstar Chaplain").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &COUNTER_SOURCE)]);
    for player in [Player::P0, Player::P1] {
        for funded in [false, true] {
            for mana_first in [false, true].into_iter().filter(|first| funded || !first) {
                let mut game = scenario(player);
                let source = spawn(&mut game, player, definition, Zone::Battlefield);
                let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
                let source_object = game.object_ref(source);
                game.add_counter(source, CounterKind::PlusOne, 2);
                game.state.players[player.index()].life = 5;
                game.state.players[player.index()].mana = if funded { add_mana(TWO) } else { ZERO };
                Game::reset_payment_planning_work();
                act(
                    &mut game,
                    Action::AnnounceActivate {
                        card: source,
                        ability: 0,
                        counter_payment: Counters::of(CounterKind::PlusOne, 1),
                    },
                );
                act(
                    &mut game,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                enter_ordered(&mut game);
                assert_mana_unpaid(&game);
                if mana_first {
                    clause(&mut game, CastCostClause::Mana);
                    same_step(&mut game, Action::PayCosts { payment: TWO });
                    assert_eq!(game.state.players[player.index()].mana, EXTRA);
                    assert_eq!(game.state.players[player.index()].life, 3);
                    assert!(game.object_view(source).tapped);
                    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
                    reject(&mut game, Action::PayCosts { payment: TWO });
                }
                clause(&mut game, CastCostClause::Additional);
                same_step(
                    &mut game,
                    Action::ChooseCreatureCounterCost {
                        object: source_object,
                        counter: CounterKind::PlusOne,
                    },
                );
                if !funded {
                    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
                    assert_eq!(game.state.players[player.index()].life, 5);
                    assert!(!game.object_view(source).tapped);
                    fail_unpaid(&mut game, player);
                    assert_eq!(game.object_ref(source), source_object);
                    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
                    assert_eq!(game.state.players[player.index()].life, 5);
                    assert!(!game.object_view(source).tapped);
                } else {
                    if !mana_first {
                        assert_mana_unpaid(&game);
                        assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
                        assert_eq!(game.state.players[player.index()].life, 5);
                        assert!(!game.object_view(source).tapped);
                        clause(&mut game, CastCostClause::Mana);
                        same_step(&mut game, Action::PayCosts { payment: TWO });
                    }
                    assert_eq!(game.state.players[player.index()].mana, EXTRA);
                    assert_eq!(game.state.players[player.index()].life, 3);
                    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
                    assert!(game.object_view(source).tapped);
                    assert_eq!(game.state.stack.len(), 1);
                    assert!(game.state.announcing.is_none());
                    settle(&mut game);
                    assert!(game.object_view(target).tapped);
                    assert_eq!(game.state.players[player.index()].life, 3);
                    assert_eq!(game.state.players[player.index()].mana, EXTRA);
                }
                assert_no_projection();
            }
        }
    }
}
