//! The damage recipient owns the target graveyard and its permanent spell offer.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PICKPOCKET: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::Single(TargetSpec::one(
            TargetRule::DamagedPlayerNonlandPermanentCard,
        )),
        program: &[Instruction::CastTargetedGraveyardPermanentPaid {
            spend_any_mana: true,
        }],
    }],
    ..CardRules::EMPTY
};

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn offered(game: &Game, card: CardId, face: CastFace) -> Action {
    Action::CastExiledDuringResolution {
        object: game.object_ref(card),
        face,
    }
}

#[test]
fn targets_include_battles_and_the_cast_uses_only_permanent_faces() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PICKPOCKET)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let otter = put(
            &mut game,
            opponent,
            "Elusive Otter // Grove's Bounty",
            Zone::Graveyard,
        );
        let land = put(&mut game, opponent, "Forest", Zone::Graveyard);
        let instant = put(&mut game, opponent, "Opt", Zone::Graveyard);
        let own = put(&mut game, player, "Llanowar Elves", Zone::Graveyard);
        let battle = spawn(
            &mut game,
            opponent,
            Card::from_name("@fixture/siege-front").unwrap(),
            Zone::Graveyard,
        );
        game.state.players[player.index()].mana = [0, 0, 0, 4, 0, 0];
        game.apply_damage_batch(&[DamageEvent {
            preventable: true,
            source: game.damage_source(source),
            target: Target::Player(opponent),
            amount: 2,
            combat: true,
        }]);
        game.prepare_triggers();
        assert!(game.legal_actions().contains(&target(battle)));
        reject(&mut game, target(land));
        reject(&mut game, target(instant));
        reject(&mut game, target(own));
        act(&mut game, target(otter));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(matches!(
            game.state.pending,
            Pending::ResolutionCasts { .. }
        ));

        let mut declined = game.clone();
        act(&mut declined, Action::FinishResolutionCasts);
        assert_eq!(declined.object_view(otter).zone, Zone::Graveyard);

        let adventure = offered(&game, otter, CastFace::Adventure);
        reject(&mut game, adventure);
        let cast = offered(&game, otter, CastFace::Normal);
        act(&mut game, cast);
        pay(&mut game);
        if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
            act(&mut game, Action::FinishResolutionCasts);
        }
        assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 3, 0, 0]);
        let spell = game.state.stack.last().unwrap();
        let cast = spell.cast.unwrap();
        assert_eq!(cast.origin, Zone::Graveyard);
        assert!(cast.spend_any_mana);
        assert!(!cast.exile_on_graveyard);
        settle(&mut game);
        let result = game.object_view(otter);
        assert_eq!(result.zone, Zone::Battlefield);
        assert_eq!(result.controller, player);
        assert_eq!(result.owner, opponent);
    }
}
