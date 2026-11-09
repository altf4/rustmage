mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const FRONT: &str = "Brigid, Clachan's Heart";
const BACK: &str = "Brigid, Doun's Mind";
const TRANSFORM: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[Instruction::TransformSource],
    sorcery: false,
    once_per_turn: false,
};
const FRONT_RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    activated: &[
        TRANSFORM,
        ActivatedAbility {
            program: &[Instruction::ExileBattlefieldSourceReturn {
                transformed: true,
                under_owner: true,
            }],
            ..TRANSFORM
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BACK_RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 8),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ChosenPlayerLosesGame,
        targets: TargetPlan::NONE,
        program: &[Instruction::WinGame],
    }],
    extras: &CardExtras {
        as_transform: Some(AsTransformEffect::ChooseOpponent),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PREVENT_LOSS: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::CantLoseGameAndOpponentsCantWin,
            StaticAbilityFragment::PlayerProtectionFromOpponents,
        ],
    }],
    ..CardRules::EMPTY
};

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name(FRONT).unwrap(), &FRONT_RULES),
        (Card::from_name(BACK).unwrap(), &BACK_RULES),
        (Card::from_name("Cathar Commando").unwrap(), &PREVENT_LOSS),
    ])
}

fn activate(game: &mut Game, source: CardId, slot: u16) {
    choose(
        game,
        |action| matches!(action, Action::Activate { card, ability, .. } if *card == source && *ability == slot),
    );
    settle(game);
}

fn assert_loss_trigger(game: &Game, source: ObjectRef, loser: Player, controller: Player) {
    assert_eq!(game.state.pending_triggers.len(), 1);
    let trigger = &game.state.pending_triggers[0];
    assert_eq!(trigger.source.unwrap().object, source);
    assert_eq!(trigger.event_player, Some(loser));
    assert_eq!(trigger.controller, controller);
    assert!(trigger.targets.is_empty());
    assert!(game.state.stack.is_empty());
    assert!(game.legal_actions().is_empty());
}

#[test]
fn the_transform_choice_is_untargeted_and_stays_with_the_same_opponent_after_control_changes() {
    let _rules = fixture();
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, FRONT, Zone::Battlefield);
    put(&mut game, opponent, "Cathar Commando", Zone::Battlefield);
    let reference = game.object_ref(source);
    activate(&mut game, source, 0);
    assert_eq!(game.object_ref(source), reference);
    assert_eq!(game.object_view(source).name(), BACK);
    let face = game.object_view(source).face.unwrap();
    assert!(face.back);
    assert_eq!(face.chosen_opponent, Some(opponent));
    game.change_controller(reference, opponent);
    assert_eq!(
        game.object_view(source).face.unwrap().chosen_opponent,
        Some(opponent)
    );

    let mut unchosen = game.clone();
    unchosen.apply(player, Action::Concede).unwrap();
    assert_eq!(unchosen.outcome(), Some(Outcome::Winner(opponent)));
    assert!(unchosen.state.pending_triggers.is_empty());

    game.apply(opponent, Action::Concede).unwrap();
    assert_eq!(game.outcome(), Some(Outcome::Winner(player)));
    assert_loss_trigger(&game, reference, opponent, opponent);
}

#[test]
fn loss_observers_look_back_before_simultaneous_death_and_ignore_draws_and_prevented_losses() {
    let _rules = fixture();
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, FRONT, Zone::Battlefield);
    let reference = game.object_ref(source);
    activate(&mut game, source, 0);
    let mut prevented = game.clone();
    put(
        &mut prevented,
        opponent,
        "Cathar Commando",
        Zone::Battlefield,
    );
    prevented.state.players[opponent.index()].life = 0;
    prevented.state_based_actions();
    assert_eq!(prevented.outcome(), None);
    assert!(prevented.state.pending_triggers.is_empty());

    let mut draw = game.clone();
    for seat in &mut draw.state.players {
        seat.life = 0;
    }
    draw.state_based_actions();
    assert_eq!(draw.outcome(), Some(Outcome::Draw));
    assert!(draw.state.pending_triggers.is_empty());

    game.state.players[opponent.index()].life = 0;
    game.state.objects[source as usize].damage = 8;
    game.state_based_actions();
    assert_eq!(game.outcome(), Some(Outcome::Winner(player)));
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_loss_trigger(&game, reference, opponent, player);
}

#[test]
fn entering_with_the_back_face_up_does_not_record_an_opponent() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, FRONT, Zone::Battlefield);
    let reference = game.object_ref(source);
    activate(&mut game, source, 1);
    assert_ne!(game.object_ref(source), reference);
    let face = game.object_view(source).face.unwrap();
    assert!(face.back);
    assert_eq!(face.transforms, 0);
    assert_eq!(face.chosen_opponent, None);
    game.apply(player.other(), Action::Concede).unwrap();
    assert_eq!(game.outcome(), Some(Outcome::Winner(player)));
    assert!(game.state.pending_triggers.is_empty());
}
