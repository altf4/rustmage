//! A spell's unlimited hand size expires when its controller's next turn begins.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const UNTIL_NEXT_TURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::NoMaximumHandSizeUntilNextTurn],
    ),
    ..CardRules::EMPTY
};
const FINITE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::MaximumHandSize(4)],
    }],
    ..CardRules::EMPTY
};
const NEWER_FINITE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::MaximumHandSize(2)],
    }],
    ..CardRules::EMPTY
};

fn end_turn(game: &mut Game, player: Player) {
    game.state.step = Step::End;
    game.state.priority = player;
    for _ in 0..12 {
        if game.state.active != player {
            return;
        }
        choose(game, |action| matches!(action, Action::Pass));
    }
    panic!("turn did not advance");
}

#[test]
fn temporary_hand_size_grant_survives_opponent_turn_then_expires() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &UNTIL_NEXT_TURN),
        (Card::from_name("Bear Cub").unwrap(), &FINITE),
        (Card::from_name("Grizzly Bears").unwrap(), &NEWER_FINITE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.grant_no_maximum_hand_size(player);
        put(&mut game, player, "Bear Cub", Zone::Battlefield);
        assert_eq!(game.maximum_hand_size(player), Some(4));

        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        settle(&mut game);
        assert_eq!(game.maximum_hand_size(player), None);
        let timeline = game.observe(player).hand_size_timeline.unwrap();
        assert!(timeline.until_next_turn[player.index()].is_some());
        assert!(timeline.lasting[player.index()].is_some());

        let newer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        assert_eq!(game.maximum_hand_size(player), Some(2));
        game.move_object(newer, Zone::Graveyard);
        assert_eq!(game.maximum_hand_size(player), None);

        for _ in 0..8 {
            put(&mut game, player, "Forest", Zone::Hand);
        }
        let checkpoint = game.snapshot();
        let mut restored = game.clone();
        restored.restore(&checkpoint).unwrap();
        let mut indexed = game.clone();
        game.state.step = Step::End;
        game.state.priority = player;
        indexed.state.step = Step::End;
        indexed.state.priority = player;
        let pass_index = indexed
            .legal_actions()
            .iter()
            .position(|action| matches!(action, Action::Pass))
            .unwrap();
        act(&mut game, Action::Pass);
        indexed
            .apply_index(player, indexed.revision(), pass_index)
            .unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());

        end_turn(&mut game, player);
        end_turn(&mut restored, player);
        assert_eq!(game.state.active, player.other());
        assert_eq!(game.maximum_hand_size(player), None);
        assert_eq!(game.hand_count(player), 8);
        assert_eq!(restored.maximum_hand_size(player), None);
        assert_eq!(
            game.observe(player).hand_size_timeline,
            restored.observe(player).hand_size_timeline
        );

        end_turn(&mut game, player.other());
        end_turn(&mut restored, player.other());
        assert_eq!(game.state.active, player);
        assert_eq!(game.maximum_hand_size(player), Some(4));
        assert_eq!(restored.maximum_hand_size(player), Some(4));
        assert_eq!(
            game.observe(player).hand_size_timeline,
            restored.observe(player).hand_size_timeline
        );
        assert_eq!(
            game.observe(player)
                .hand_size_timeline
                .unwrap()
                .until_next_turn[player.index()],
            None
        );
    }
}
