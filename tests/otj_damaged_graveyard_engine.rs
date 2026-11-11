//! Tinybones binds an exact permanent card, then offers its legal spell faces.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn cast_offered_face(game: &mut Game, card: CardId, face: CastFace, targets: &[CardId]) {
    let object = game.object_ref(card);
    act(game, Action::CastExiledDuringResolution { object, face });
    for &id in targets {
        act(game, target(id));
    }
    pay(game);
    if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
        act(game, Action::FinishResolutionCasts);
    }
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.face, face);
    assert_eq!(cast.origin, Zone::Graveyard);
    assert!(cast.spend_any_mana);
    assert!(!cast.exile_on_graveyard);
}

#[test]
fn damaged_player_targets_include_battles_and_allow_a_paid_adventure_face() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let source = put(
            &mut game,
            player,
            "Tinybones, the Pickpocket",
            Zone::Battlefield,
        );
        let virtue = put(
            &mut game,
            opponent,
            "Virtue of Persistence // Locthwain Scorn",
            Zone::Graveyard,
        );
        let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let land = put(&mut game, opponent, "Forest", Zone::Graveyard);
        let instant = put(&mut game, opponent, "Opt", Zone::Graveyard);
        let own = put(&mut game, player, "Llanowar Elves", Zone::Graveyard);
        let battle = spawn(
            &mut game,
            opponent,
            Card::from_name("@fixture/siege-front").unwrap(),
            Zone::Graveyard,
        );
        game.state.players[player.index()].mana = [0, 0, 0, 7, 0, 0];
        game.apply_damage_batch(&[DamageEvent {
            preventable: true,
            source: game.damage_source(source),
            target: Target::Player(opponent),
            amount: 1,
            combat: true,
        }]);
        game.prepare_triggers();
        assert!(game.legal_actions().contains(&target(battle)));
        for card in [land, instant, own] {
            reject(&mut game, target(card));
        }
        act(&mut game, target(virtue));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(matches!(
            game.state.pending,
            Pending::ResolutionCasts { .. }
        ));

        let mut declined = game.clone();
        act(&mut declined, Action::FinishResolutionCasts);
        assert_eq!(declined.object_view(virtue).zone, Zone::Graveyard);

        let mut adventure = game.clone();
        cast_offered_face(&mut adventure, virtue, CastFace::Adventure, &[victim]);
        assert_eq!(
            adventure.state.players[player.index()].mana,
            [0, 0, 0, 5, 0, 0]
        );
        settle(&mut adventure);
        assert_eq!(adventure.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(adventure.object_view(virtue).zone, Zone::Exile);
        assert_eq!(adventure.state.players[player.index()].life, 22);

        cast_offered_face(&mut game, virtue, CastFace::Normal, &[]);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        settle(&mut game);
        let result = game.object_view(virtue);
        assert_eq!(result.zone, Zone::Battlefield);
        assert_eq!(result.controller, player);
        assert_eq!(result.owner, opponent);
    }
}
