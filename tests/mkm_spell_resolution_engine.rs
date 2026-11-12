//! Owned spell-resolution packets for grouped copies and reflexive effects.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const MULTIPLE_TARGET_COPIES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: usize::MAX,
        }),
        &[
            Instruction::TargetsExactlyX,
            Instruction::CreateCopiesOfEachTarget(AmountValue::ChosenX),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 3,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture {name}"))
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn copies(game: &Game, player: Player, name: &str) -> Vec<ObjectView> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            if object.zone != Zone::Battlefield || object.controller != player {
                return None;
            }
            let view = game.object_view(id as CardId);
            (view.is_token() && view.card == named(name)).then_some(view)
        })
        .collect()
}

#[test]
fn each_legal_target_contributes_the_declared_number_to_one_token_batch() {
    let _rules = fixture_rules::activate(&[(named("Divination"), &MULTIPLE_TARGET_COPIES)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let doubler = put(&mut game, player, "Doubling Season", Zone::Battlefield);
        let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 12];
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(&mut game, Action::ChooseX { value: 3 });

        let mut zero = game.clone();
        act(&mut zero, Action::ChooseX { value: 0 });
        pay(&mut zero);
        settle(&mut zero);
        assert!(copies(&zero, player, "Doubling Season").is_empty());
        assert!(copies(&zero, player, "Forest").is_empty());

        let choice = Action::ChooseX { value: 2 };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == choice)
            .unwrap();
        let saved = game.snapshot();
        let revision = game.revision();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, choice.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, choice);
        assert_eq!(game.snapshot(), indexed.snapshot());
        reject(&mut game, Action::FinishTargets);
        act(&mut game, target(doubler));
        reject(&mut game, target(doubler));
        act(&mut game, target(land));
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        assert_eq!(game.mana_payment_domain().unwrap().generic, 8);
        pay(&mut game);
        assert_eq!(game.state.stack.last().unwrap().mana_value(), 9);
        settle(&mut game);
        assert_eq!(copies(&game, player, "Forest").len(), 4);
        assert_eq!(copies(&game, player, "Doubling Season").len(), 4);
    }
}

#[test]
fn partial_target_failure_preserves_x_and_copies_only_the_live_copiable_values() {
    let _rules = fixture_rules::activate(&[(named("Divination"), &MULTIPLE_TARGET_COPIES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let copy = CopyRecipe::new(named("Grizzly Bears"));
    game.apply_temporary_copy(game.object_ref(creature), copy, CopyDeadline::EndOfTurn);
    game.add_counter(creature, CounterKind::PlusOne, 2);
    game.state.objects[creature as usize].tapped = true;
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseX { value: 2 });
    act(&mut game, target(creature));
    act(&mut game, target(land));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);

    let original_land = game.object_ref(land);
    game.move_object(land, Zone::Hand);
    game.move_object(land, Zone::Battlefield);
    assert_ne!(game.object_ref(land), original_land);
    let mut failed = game.clone();
    failed.move_object(creature, Zone::Hand);
    settle(&mut failed);
    assert!(copies(&failed, player, "Grizzly Bears").is_empty());

    settle(&mut game);
    assert!(copies(&game, player, "Forest").is_empty());
    let tokens = copies(&game, player, "Grizzly Bears");
    assert_eq!(tokens.len(), 2);
    for token in tokens {
        assert!(!token.tapped);
        assert_eq!(token.counters.get(CounterKind::PlusOne), 0);
        assert_eq!((token.power, token.toughness), (2, 2));
    }
}
