//! Holo-Engineer's replacement changes every group in a token creation event.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SPIRIT: Card = Card::from_catalogue_id(5326);
const FOOD: Card = Card::from_catalogue_id(5246);
const GNOME: Card = Card::from_catalogue_id(5267);
const SUBSTITUTE: ReplacementSpec = ReplacementSpec::ReplaceTokens(Card::DrakeToken);
const ENGINEER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, SUBSTITUTE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[SUBSTITUTE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MIXED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokenBatch(&[
            TokenGroupView {
                token: SPIRIT,
                amount: 2,
                tapped: false,
            },
            TokenGroupView {
                token: FOOD,
                amount: 2,
                tapped: true,
            },
            TokenGroupView {
                token: GNOME,
                amount: 1,
                tapped: false,
            },
        ])],
    ),
    ..CardRules::EMPTY
};

fn tokens(game: &Game, player: Player) -> Vec<ObjectView> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.zone == Zone::Battlefield && object.is_token() && object.controller == player
        })
        .map(|(id, _)| game.object_view(id as CardId))
        .collect()
}

#[test]
fn all_token_recipes_are_substituted_with_group_quantities_and_entry_flags() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &ENGINEER),
        (Card::from_name("Opt").unwrap(), &MIXED),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let provider = put(&mut game, player.other(), "Grizzly Bears", Zone::Battlefield);
        let original = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, original, None);
        settle(&mut game);
        assert_eq!(tokens(&game, player).len(), 5);
        assert!(tokens(&game, player)
            .iter()
            .all(|token| token.card != Card::DrakeToken));

        game.record_control(game.object_ref(provider), player, None);
        let before = game.snapshot();
        let substituted = put(&mut game, player, "Opt", Zone::Hand);
        let action = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::Cast { card, .. } if *card == substituted))
            .unwrap();
        let index = game.legal_actions().iter().position(|a| *a == action).unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(player, game.revision(), index).unwrap();
        act(&mut game, action);
        settle(&mut game);
        settle(&mut indexed);
        assert_eq!(game.snapshot(), indexed.snapshot());
        let created: Vec<_> = tokens(&game, player)
            .into_iter()
            .filter(|token| token.card == Card::DrakeToken)
            .collect();
        assert_eq!(created.len(), 5);
        assert_eq!(created.iter().filter(|token| token.tapped).count(), 2);

        let mut restored = game.clone();
        restored.restore(&before).unwrap();
        assert_eq!(tokens(&restored, player).len(), 5);
        assert!(tokens(&restored, player)
            .iter()
            .all(|token| token.card != Card::DrakeToken));
    }
}
