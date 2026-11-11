//! A resolving permanent can acquire a live static ability granting tap mana.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantPermanentSelfAbility(
            SelfGrantedAbilityKind::ControlledPermanentTapMana {
                kinds: 1,
                colors: 1 | 8 | 16,
            },
        )],
    }],
    ..CardRules::EMPTY
};

fn provider_rules() -> impl Drop {
    fixture_rules::activate(&[(Card::from_name("Swiftfoot Boots").unwrap(), &PROVIDER)])
}

fn cast_provider(game: &mut Game, player: Player) -> CardId {
    let source = put(game, player, "Swiftfoot Boots", Zone::Hand);
    cast(game, source, None);
    settle(game);
    source
}

#[test]
fn acquired_grant_funds_a_spell_from_a_previously_nonproducing_creature() {
    let _rules = provider_rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let opponent = player.other();
        let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let foe = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let new = put(&mut game, player, "Savannah Lions", Zone::Hand);
        cast(&mut game, new, None);
        settle(&mut game);
        assert!(CharacteristicWorld::granted_mana(&game, recipient).is_empty());
        let source = cast_provider(&mut game, player);
        assert!(CharacteristicWorld::granted_mana(&game, foe).is_empty());
        assert_eq!(CharacteristicWorld::granted_mana(&game, recipient).len(), 1);
        game.state.players[player.index()].mana = [0; 6];
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: shock });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(opponent),
            },
        );
        let grantor = game.object_ref(source);
        let red = [0, 0, 0, 1, 0, 0];
        reject(
            &mut game,
            Action::TapGrantedMana {
                card: recipient,
                grantor,
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        reject(
            &mut game,
            Action::TapGrantedMana {
                card: new,
                grantor,
                production: red,
            },
        );
        let action = Action::TapGrantedMana {
            card: recipient,
            grantor,
            production: red,
        };
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let mut indexed = game.clone();
        act(&mut game, action);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE - 2);
        assert!(game.object_view(recipient).tapped);
    }
}

#[test]
fn acquired_static_grant_follows_live_control_and_ability_order_and_ends_on_departure() {
    let _rules = provider_rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let foe = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let source = cast_provider(&mut game, player);
    let exact = game.object_ref(source);
    let checkpoint = game.snapshot();

    game.change_controller(exact, player.other());
    assert!(CharacteristicWorld::granted_mana(&game, own).is_empty());
    assert_eq!(CharacteristicWorld::granted_mana(&game, foe).len(), 1);

    game.restore(&checkpoint).unwrap();
    game.remove_abilities_exact(&[exact], exact, player, AnimationDuration::EndOfTurn);
    assert!(CharacteristicWorld::granted_mana(&game, own).is_empty());

    game.restore(&checkpoint).unwrap();
    game.remove_abilities_exact(
        &[game.object_ref(own)],
        exact,
        player,
        AnimationDuration::EndOfTurn,
    );
    assert!(CharacteristicWorld::granted_mana(&game, own).is_empty());

    game.restore(&checkpoint).unwrap();
    game.move_object(source, Zone::Graveyard);
    assert!(CharacteristicWorld::granted_mana(&game, own).is_empty());
    assert!(game.observe(player).self_granted_abilities.is_none());
}
