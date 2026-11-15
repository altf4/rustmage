//! Dominion Infiltration's real card wiring: X, opposing targets, and exact copies.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn tokens(game: &Game, player: Player) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.controller == player && object.is_token())
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn paid_x_selects_only_opposing_creatures_and_exiles_their_hasty_changelings() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let bear = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let giant = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, player, "Dominion Infiltration", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });

        let mut zero = game.clone();
        act(&mut zero, Action::ChooseX { value: 0 });
        pay(&mut zero);
        settle(&mut zero);
        assert!(tokens(&zero, player).is_empty());

        act(&mut game, Action::ChooseX { value: 2 });
        reject(&mut game, Action::FinishTargets);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(giant),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        assert_eq!(game.mana_payment_domain().unwrap().generic, 2);
        pay(&mut game);
        assert_eq!(game.state.stack.last().unwrap().mana_value(), 5);
        settle(&mut game);
        let created = tokens(&game, player);
        assert_eq!(created.len(), 2);
        assert!(
            created
                .iter()
                .any(|&id| game.object_view(id).card == Card::from_name("Bear Cub").unwrap())
        );
        assert!(
            created
                .iter()
                .any(|&id| game.object_view(id).card == Card::from_name("Hill Giant").unwrap())
        );
        for &id in &created {
            assert!(
                game.keywords(id)
                    .contains(Keywords::CHANGELING.union(Keywords::HASTE))
            );
            assert!(game.has_subtype(id, "Vulcan"));
        }
        assert_eq!(game.observe(player).delayed_steps.unwrap().len(), 1);
        game.begin_step(Step::End);
        game.prepare_triggers();
        settle(&mut game);
        assert!(
            created
                .into_iter()
                .all(|id| game.object_view(id).zone != Zone::Battlefield)
        );
        assert_eq!(game.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(game.object_view(giant).zone, Zone::Battlefield);
    }
}
