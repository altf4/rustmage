//! The first optional instruction does not gate the following non-targeted
//! choice. A partial selection returns exactly the cards selected.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayMill(2),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnUpToGraveyardToHand { kinds: 1 },
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 2,
                optional_skip: None,
            },
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn optional_mill_and_up_to_two_graveyard_cards_are_independent() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &RETURN)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let old = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        let other = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let ineligible = put(&mut game, player, "Forest", Zone::Graveyard);
        let foreign = put(&mut game, player.other(), "Savannah Lions", Zone::Graveyard);
        let milled = put(&mut game, player, "Hill Giant", Zone::Library);
        put(&mut game, player, "Forest", Zone::Library);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        while matches!(game.state.pending, Pending::Priority) {
            act(&mut game, Action::Pass);
        }
        assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));
        let checkpoint = game.snapshot();
        act(&mut game, Action::ChooseOptional { accept: false });
        assert_eq!(game.object_view(milled).zone, Zone::Library);
        for card in [ineligible, foreign, milled] {
            let object = game.object_ref(card);
            reject(&mut game, Action::SelectGraveyardReturn { object });
        }
        let object = game.object_ref(old);
        act(&mut game, Action::SelectGraveyardReturn { object });
        assert_eq!(game.object_view(old).zone, Zone::Graveyard);
        act(&mut game, Action::FinishGraveyardReturn);
        settle(&mut game);
        assert_eq!(game.object_view(old).zone, Zone::Hand);
        assert_eq!(game.object_view(other).zone, Zone::Graveyard);

        game.restore(&checkpoint).unwrap();
        act(&mut game, Action::ChooseOptional { accept: true });
        assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
        for card in [old, milled] {
            let object = game.object_ref(card);
            act(&mut game, Action::SelectGraveyardReturn { object });
        }
        settle(&mut game);
        assert_eq!(game.object_view(milled).zone, Zone::Hand);
        assert_eq!(game.object_view(old).zone, Zone::Hand);
        assert_eq!(game.object_view(other).zone, Zone::Graveyard);

        game.restore(&checkpoint).unwrap();
        act(&mut game, Action::ChooseOptional { accept: false });
        act(&mut game, Action::FinishGraveyardReturn);
        settle(&mut game);
        assert_eq!(game.object_view(old).zone, Zone::Graveyard);
        assert_eq!(game.object_view(other).zone, Zone::Graveyard);
    }
}
