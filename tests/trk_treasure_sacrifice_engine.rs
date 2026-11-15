//! Quark's static rule restricts the sacrifice action, including Treasure mana.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::*;
use support::*;

const TREASURE: Card = Card::from_catalogue_id(5302);
const QUARK_RULE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledTreasureTokensCannotBeSacrificed],
    }],
    ..CardRules::EMPTY
};

fn treasure(game: &mut Game, player: Player) -> CardId {
    let before = game.state.objects.len();
    game.create_tokens(player, TREASURE, 1, false);
    (before..game.state.objects.len())
        .find(|&id| game.object_view(id as CardId).has_subtype("Treasure"))
        .unwrap() as CardId
}

#[test]
fn quark_rule_blocks_only_controlled_treasure_token_sacrifices() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Bear Cub").unwrap(), &QUARK_RULE)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let quark = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let ours = treasure(&mut game, player);
        let theirs = treasure(&mut game, player.other());
        let other_artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
        assert!(CharacteristicWorld::sacrifice_prohibited(&game, ours));
        assert!(!CharacteristicWorld::sacrifice_prohibited(&game, theirs));
        assert!(!CharacteristicWorld::sacrifice_prohibited(&game, other_artifact));
        assert!(!game.legal_actions().iter().any(|action| {
            matches!(action, Action::ActivateMana { card, .. } if *card == ours)
        }));

        let checkpoint = game.snapshot();
        game.move_object(quark, Zone::Graveyard);
        assert!(!CharacteristicWorld::sacrifice_prohibited(&game, ours));
        let mana = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::ActivateMana { card, .. } if *card == ours))
            .unwrap();
        act(&mut game, mana);
        assert_ne!(game.object_view(ours).zone, Zone::Battlefield);

        game.restore(&checkpoint).unwrap();
        assert!(CharacteristicWorld::sacrifice_prohibited(&game, ours));
        game.record_control(game.object_ref(quark), player.other(), None);
        assert!(!CharacteristicWorld::sacrifice_prohibited(&game, ours));
        assert!(CharacteristicWorld::sacrifice_prohibited(&game, theirs));
    }
}
