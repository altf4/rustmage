//! Private hand inspection, card revelation and simultaneous deployment.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OPPONENT: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Player {
    opponent_only: true,
}));
const INSPECT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: OPPONENT,
        program: &[Instruction::LookTargetHandMayExileUntilSourceLeaves],
    }],
    ..CardRules::EMPTY
};
const EXILE: CardRules = CardRules {
    effect: Effect::Spell(OPPONENT, &[Instruction::TargetPlayerExilesHandCard]),
    ..CardRules::EMPTY
};
const REVEAL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayRevealHandSubtype {
                subtype: "Dinosaur",
                skip: 2,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
            Instruction::Skip(2),
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Dinosaur"),
                    basic: false,
                    other: true,
                    cards_only: false,
                })),
                minimum: 1,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    ),
    ..CardRules::EMPTY
};
const DEPLOY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DeployControllerHandCreatures],
    ),
    ..CardRules::EMPTY
};

fn hand_card(game: &Game, card: CardId) -> Action {
    Action::ChooseHandCard {
        object: Some(game.object_ref(card)),
    }
}

fn advance_to_hand(game: &mut Game, target: Option<Player>) {
    for _ in 0..24 {
        if matches!(game.state.pending, Pending::HandCardChoice { .. }) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
                || matches!(action, Action::ChooseTarget { target: Target::Player(player) }
                    if Some(*player) == target)
        });
    }
    panic!("hand choice did not open");
}

#[test]
fn target_hand_is_private_and_only_a_live_source_can_exile_a_nonland() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &INSPECT)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let victim = player.other();
        let nonland = put(&mut game, victim, "Grizzly Bears", Zone::Hand);
        let land = put(&mut game, victim, "Forest", Zone::Hand);
        let source = put(&mut game, player, "Hill Giant", Zone::Hand);
        cast(&mut game, source, None);
        for _ in 0..4 {
            if game.object_view(source).zone == Zone::Battlefield {
                break;
            }
            act(&mut game, Action::Pass);
        }
        if matches!(game.state.pending, Pending::Targets { .. }) {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Player(victim),
                },
            );
        }
        let mut departed = game.clone();
        departed.move_object(source, Zone::Exile);
        advance_to_hand(&mut departed, Some(victim));
        let view = departed.observe(player).hand_choice.unwrap();
        assert_eq!(view.inspected_cards.unwrap().len(), 2);
        assert!(view.candidates.is_empty());
        act(&mut departed, Action::ChooseHandCard { object: None });
        assert_eq!(departed.object_view(nonland).zone, Zone::Hand);

        advance_to_hand(&mut game, Some(victim));
        let view = game.observe(player).hand_choice.unwrap();
        assert_eq!(view.actor, player);
        assert_eq!(view.candidates, [game.object_ref(nonland)]);
        assert_eq!(view.inspected_cards.unwrap().len(), 2);
        assert!(game.observe(victim).hand_choice.is_none());
        assert!(game.observe(player).known_hand.is_empty());
        let land_action = hand_card(&game, land);
        reject(&mut game, land_action);

        let action = hand_card(&game, nonland);
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let mut indexed = game.clone();
        act(&mut game, action.clone());
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.object_view(nonland).zone, Zone::Exile);
        game.move_object(source, Zone::Graveyard);
        settle(&mut game);
        assert_eq!(game.object_view(nonland).zone, Zone::Hand);
    }
}

#[test]
fn victim_can_exile_a_land_without_disclosing_other_hand_cards() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &EXILE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let victim = player.other();
    let nonland = put(&mut game, victim, "Grizzly Bears", Zone::Hand);
    let land = put(&mut game, victim, "Forest", Zone::Hand);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(victim)));
    advance_to_hand(&mut game, None);
    assert_eq!(game.actor(), Some(victim));
    assert!(game.observe(player).hand_choice.is_none());
    assert!(!game.observe(victim).hand_choice.unwrap().optional);
    reject(&mut game, Action::ChooseHandCard { object: None });
    let action = hand_card(&game, land);
    act(&mut game, action);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(nonland).zone, Zone::Hand);
    assert!(game.observe(player).known_hand.is_empty());
}

#[test]
fn optional_reveal_uses_only_hand_and_fallback_checks_a_controlled_permanent() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &REVEAL)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let board = put(
        &mut game,
        player,
        "Ghalta, Primal Hunger",
        Zone::Battlefield,
    );
    let hand = put(&mut game, player, "Ghalta, Primal Hunger", Zone::Hand);
    let private = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    advance_to_hand(&mut game, None);
    let snapshot = game.snapshot();
    let action = hand_card(&game, board);
    reject(&mut game, action);
    let action = hand_card(&game, private);
    reject(&mut game, action);
    assert!(game.observe(player.other()).hand_choice.is_none());

    let action = hand_card(&game, hand);
    act(&mut game, action);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 3);
    assert_eq!(
        game.observe(player.other()).known_hand,
        [game.object_ref(hand)]
    );

    game.restore(&snapshot).unwrap();
    act(&mut game, Action::ChooseHandCard { object: None });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 3);
    assert!(game.observe(player.other()).known_hand.is_empty());

    game.restore(&snapshot).unwrap();
    game.move_object(board, Zone::Exile);
    act(&mut game, Action::ChooseHandCard { object: None });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE);
}

#[test]
fn one_controller_deploys_only_own_creatures_in_one_batch() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DEPLOY)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let second = put(&mut game, player, "Savannah Lions", Zone::Hand);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Hand);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    advance_to_hand(&mut game, None);
    let artifact_action = hand_card(&game, artifact);
    reject(&mut game, artifact_action);
    let foreign_action = hand_card(&game, foreign);
    reject(&mut game, foreign_action);
    for card in [first, second] {
        let action = hand_card(&game, card);
        act(&mut game, action);
    }
    assert!(game.observe(player.other()).hand_choice.is_none());
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    act(&mut game, Action::ChooseHandCard { object: None });
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(game.object_view(artifact).zone, Zone::Hand);
    assert_eq!(game.object_view(foreign).zone, Zone::Hand);
}
