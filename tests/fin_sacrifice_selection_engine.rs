mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ATTACKER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MaySacrificeAnyNumber(SacrificeFilter {
                kinds: 1,
                other: true,
                keywords: Keywords::NONE,
                source_only: false,
                nonland: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            }),
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::MovedCount),
        ],
    }],
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDies {
            relation: CastRelation::You,
            other: true,
            nontoken: false,
            subtype: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name("Hill Giant").unwrap(), &ATTACKER),
        (Card::from_name("Bear Cub").unwrap(), &OBSERVER),
    ])
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, attacker: CardId) {
    pass_priority(game);
    pass_priority(game);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    assert_eq!(game.state.stack.len(), 1);
    pass_priority(game);
}

fn select(game: &mut Game, id: CardId) {
    act(
        game,
        Action::Sacrifice {
            object: game.object_ref(id),
        },
    );
}

#[test]
fn selection_can_finish_at_zero_or_commit_one_simultaneous_batch() {
    let _rules = fixture();
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let observer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let third = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let opposing = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
    attack(&mut game, source);
    let choice = game.observe(opponent).effect_choice.unwrap();
    assert!(choice.optional);
    assert_eq!(choice.required, [0, 0]);
    assert_eq!(choice.amount, 3);
    for id in [source, opposing, equipment] {
        let object = game.object_ref(id);
        reject(&mut game, Action::Sacrifice { object });
    }
    let before_selection = game.snapshot();
    select(&mut game, observer);
    select(&mut game, other);
    for id in [observer, other] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
        let object = game.object_ref(id);
        reject(&mut game, Action::Sacrifice { object });
    }
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let finish = Action::FinishEffectChoice;
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == finish)
        .unwrap();
    let saved = game.snapshot();
    let revision = game.revision();
    let mut indexed = game.clone();
    assert_eq!(game.apply(opponent, finish.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), saved);
    act(&mut game, finish.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(observer).zone, Zone::Graveyard);
    assert_eq!(game.object_view(other).zone, Zone::Graveyard);
    assert_eq!(game.object_view(third).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(
        game.state.players[player.index()].permanents_sacrificed_this_turn,
        2
    );
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].source.unwrap().id, observer);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);

    indexed.restore(&before_selection).unwrap();
    act(&mut indexed, finish);
    assert_eq!(
        indexed.state.players[player.index()].cards_drawn_this_turn,
        0
    );
    assert_eq!(indexed.object_view(observer).zone, Zone::Battlefield);
    assert_eq!(indexed.object_view(other).zone, Zone::Battlefield);
}

#[test]
fn exile_replacement_counts_and_a_prevented_move_does_not() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let _reset = Reset;
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let exiled = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let prevented = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    game.add_counter(exiled, CounterKind::Finality, 1);
    zone_replacements::TEST_PREVENT_ZONE_MOVE
        .with(|rule| rule.set(Some(game.object_ref(prevented))));
    attack(&mut game, source);
    select(&mut game, exiled);
    select(&mut game, prevented);
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.object_view(exiled).zone, Zone::Exile);
    assert_eq!(game.object_view(prevented).zone, Zone::Battlefield);
    assert_eq!(
        game.state.players[player.index()].permanents_sacrificed_this_turn,
        1
    );
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn no_matching_permanent_skips_the_choice_and_draw() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    attack(&mut game, source);
    assert!(game.observe(player).effect_choice.is_none());
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
}
