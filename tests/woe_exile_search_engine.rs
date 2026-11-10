//! The target's battlefield name and controller bind a private resolution search.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXILE_SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::ExileTargetAndSearchItsController],
    ),
    ..CardRules::EMPTY
};

#[test]
fn exile_search_uses_the_pre_exile_controller_and_draws_for_actual_hand_exiles() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Murder").unwrap(), &EXILE_SEARCH)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let other = player.other();
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.state.objects[creature as usize].controller = other;
        let own_match = put(&mut game, player, "Bear Cub", Zone::Hand);
        let hand_match = put(&mut game, other, "Bear Cub", Zone::Hand);
        let grave_match = put(&mut game, other, "Bear Cub", Zone::Graveyard);
        let library_match = put(&mut game, other, "Bear Cub", Zone::Library);
        let other_name = put(&mut game, other, "Hill Giant", Zone::Hand);
        let spell = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);

        assert_eq!(game.object_view(creature).zone, Zone::Exile);
        assert_eq!(game.actor(), Some(player));
        assert!(game.observe(player).effect_choice.is_some());
        assert!(game.observe(other).effect_choice.is_none());
        for id in [own_match, other_name] {
            assert!(
                !game
                    .legal_actions()
                    .contains(&Action::SelectResolutionExile {
                        object: game.object_ref(id),
                    })
            );
        }
        for id in [hand_match, grave_match, library_match] {
            let object = game.object_ref(id);
            act(&mut game, Action::SelectResolutionExile { object });
        }

        let checkpoint = game.snapshot();
        let revision = game.revision();
        let finish = game
            .legal_actions()
            .iter()
            .position(|a| *a == Action::FinishSearch)
            .unwrap();
        let mut index = game.clone();
        index.apply_index(player, revision, finish).unwrap();
        act(&mut game, Action::FinishSearch);
        assert_eq!(game.snapshot(), index.snapshot());
        assert_eq!(game.state.players[other.index()].cards_drawn_this_turn, 1);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
        for id in [hand_match, grave_match, library_match] {
            assert_eq!(game.object_view(id).zone, Zone::Exile);
        }
        assert_eq!(game.object_view(own_match).zone, Zone::Hand);
        assert_eq!(game.object_view(other_name).zone, Zone::Hand);
        index.restore(&checkpoint).unwrap();
        act(&mut index, Action::FinishSearch);
        assert_eq!(game.snapshot(), index.snapshot());
    }
}
