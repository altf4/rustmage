//! Resolution-time donations, live ownership counts, and exact opposing reentry.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ALLY: Card = Card::from_catalogue_id(5272);
const IROH: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: true,
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::MayIfTargetGroupsPresent {
                    groups: &[0, 1],
                    skip: 3,
                },
                Instruction::GainControlRememberCount {
                    subject: Subject::Targets,
                    controller: PlayerSet::Targeted,
                },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::MovedCount,
                    minimum: 1,
                    skip: 1,
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensThenCounters {
                token: ALLY,
                amount: 1,
                kind: CounterKind::PlusOne,
                counters: AmountValue::Stat(StatValue::OwnedPermanentsOpponentsControl),
            }],
        },
    ],
    ..CardRules::EMPTY
};
const ZUKO: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOnce(&[
            SpellMode {
                text: "Draw a card.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::Draw(PlayerSet::Controller, 1),
                    Instruction::LoseLife(PlayerSet::Controller, 2),
                ],
            },
            SpellMode {
                text: "Exile the source and return it under an opponent's control.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::ExileBattlefieldSourceReturnToOpponent,
                    Instruction::LoseLife(PlayerSet::Controller, 2),
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn allies(game: &Game) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.card == ALLY).then_some(id as CardId)
        })
        .collect()
}

#[test]
fn donation_needs_both_targets_and_counters_each_created_token_from_live_ownership() {
    let _rules = fixture_rules::activate(&[(named("Hill Giant"), &IROH)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let older_gift = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let older_ref = game.object_ref(older_gift);
    let source_ref = game.object_ref(source);
    game.record_control(older_ref, opponent, None);
    put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);

    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    let mut invalid_player = game.clone();
    invalid_player.state.players[opponent.index()].hexproof_until_end_turn = true;
    settle(&mut invalid_player);
    assert!(allies(&invalid_player).is_empty());
    assert_eq!(invalid_player.object_view(source).controller, player);

    let mut invalid_permanent = game.clone();
    invalid_permanent.record_control(source_ref, opponent, None);
    settle(&mut invalid_permanent);
    assert!(allies(&invalid_permanent).is_empty());

    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.object_view(source).controller, player);
    assert!(allies(&decline).is_empty());

    let acceptance = Action::ChooseOptional { accept: true };
    let observation = game.observe(player);
    let index = observation
        .actions
        .iter()
        .position(|action| *action == acceptance)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, observation.revision, index)
        .unwrap();
    act(&mut game, acceptance);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_ref(source), source_ref);
    assert_eq!(game.object_view(source).controller, opponent);
    assert_eq!(game.state.stack.last().unwrap().controller, player);
    assert!(allies(&game).is_empty());

    game.move_object(older_gift, Zone::Exile);
    settle(&mut game);
    let tokens = allies(&game);
    assert_eq!(tokens.len(), 2);
    for token in tokens {
        let view = game.object_view(token);
        assert_eq!(view.controller, player);
        assert_eq!(game.counter_count(token, CounterKind::PlusOne), 2);
    }
}

fn finish_turn(game: &mut Game) {
    game.state.step = Step::End;
    game.give_priority();
    pass_pair(game);
    assert_eq!(game.state.step, Step::Upkeep);
}

fn enter_first_main(game: &mut Game) {
    pass_pair(game);
    pass_pair(game);
    assert_eq!(game.state.step, Step::Main1);
}

#[test]
fn opposing_reentry_moves_only_the_original_source_and_resets_modal_history() {
    let _rules = fixture_rules::activate(&[(named("Goblin Piker"), &ZUKO)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let original = game.object_ref(source);
    game.add_counter(source, CounterKind::PlusOne, 1);
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    let draw = Action::ChooseMode { mode: 0 };
    act(&mut game, draw.clone());
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);

    finish_turn(&mut game);
    finish_turn(&mut game);
    assert_eq!(game.state.active, player);
    enter_first_main(&mut game);
    reject(&mut game, draw.clone());
    act(&mut game, Action::ChooseMode { mode: 1 });

    let mut replaced = game.clone();
    replaced.move_object(source, Zone::Hand);
    replaced.move_object(source, Zone::Battlefield);
    let later = replaced.object_ref(source);
    settle(&mut replaced);
    assert_eq!(replaced.object_ref(source), later);
    assert_eq!(replaced.object_view(source).controller, player);
    assert_eq!(replaced.state.players[player.index()].life, 16);

    settle(&mut game);
    let returned = game.object_view(source);
    assert_ne!(returned.reference, original);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.controller, opponent);
    assert_eq!(returned.owner, player);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
    assert_eq!(game.state.players[player.index()].life, 16);

    finish_turn(&mut game);
    assert_eq!(game.state.active, opponent);
    enter_first_main(&mut game);
    let before = game.state.players[opponent.index()].cards_drawn_this_turn;
    act(&mut game, draw);
    settle(&mut game);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        before + 1
    );
    assert_eq!(game.state.players[opponent.index()].life, 18);
}
