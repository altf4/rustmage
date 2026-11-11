//! One cast-entry trigger sacrifices a simultaneous cohort, then stages a
//! separate simultaneous return under the eligible cards' owners' control.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RETURN: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::SacrificeOtherCreaturesRememberGraveyard,
            Instruction::ReturnGraveyardCreaturesToOwnersExceptRemembered,
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn all_other_creatures_are_sacrificed_and_only_prior_graveyard_cards_return() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &RETURN)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let opponent = player.other();
        let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let stolen = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.change_controller(game.object_ref(stolen), opponent);
        let finality = put(&mut game, opponent, "Serra Angel", Zone::Battlefield);
        game.add_counter(finality, CounterKind::Finality, 1);
        let own_grave = put(&mut game, player, "Savannah Lions", Zone::Graveyard);
        let foreign_grave = put(&mut game, opponent, "Gleaming Barrier", Zone::Graveyard);
        let land = put(&mut game, opponent, "Forest", Zone::Graveyard);
        let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        let source = put(&mut game, player, "Hill Giant", Zone::Hand);
        cast(&mut game, source, None);
        let checkpoint = game.snapshot();
        settle(&mut game);
        let completed = game.snapshot();
        for id in [own, stolen, land] {
            assert_eq!(game.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(game.object_view(finality).zone, Zone::Exile);
        for (id, controller) in [
            (own_grave, player),
            (foreign_grave, opponent),
            (source, player),
            (equipment, player),
        ] {
            let object = game.object_view(id);
            assert_eq!(object.zone, Zone::Battlefield);
            assert_eq!(object.controller, controller);
        }
        game.restore(&checkpoint).unwrap();
        settle(&mut game);
        assert_eq!(game.snapshot(), completed);
    }
}

#[test]
fn reentry_does_not_trigger_and_does_not_keep_the_old_sources_exception() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &RETURN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Hand);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let waiting = put(&mut game, player, "Savannah Lions", Zone::Graveyard);
    let mut noncast = game.clone();
    noncast.move_object(source, Zone::Battlefield);
    settle(&mut noncast);
    assert_eq!(noncast.object_view(own).zone, Zone::Battlefield);
    assert_eq!(noncast.object_view(waiting).zone, Zone::Graveyard);

    cast(&mut game, source, None);
    for _ in 0..4 {
        if game.object_view(source).zone == Zone::Battlefield {
            break;
        }
        act(&mut game, Action::Pass);
    }
    let first = game.object_ref(source);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(game.state.stack.len(), 1);
    game.move_object(source, Zone::Exile);
    game.move_object(source, Zone::Battlefield);
    assert_ne!(game.object_ref(source), first);
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.object_view(waiting).zone, Zone::Battlefield);
}
