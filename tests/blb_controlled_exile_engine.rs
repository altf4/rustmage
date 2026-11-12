//! A non-targeted controlled exile choice binds the exact chosen permanent.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CHOOSE_EXILE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::ExileControlled,
            players: PlayerSet::Targeted,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 32,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
            amount: 1,
            optional_skip: None,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn targeted_player_chooses_an_exact_controlled_enchantment_to_exile() {
    let definition = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &CHOOSE_EXILE)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let spell = spawn(&mut game, player, definition, Zone::Hand);
        let creature = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let ineligible = put(&mut game, player, "Impact Tremors", Zone::Battlefield);
        let enchantment = put(&mut game, opponent, "Impact Tremors", Zone::Battlefield);
        let other = put(&mut game, opponent, "Impact Tremors", Zone::Battlefield);
        game.state.objects[enchantment as usize].gained_keywords = Keywords::HEXPROOF;
        cast(&mut game, spell, Some(Target::Player(opponent)));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.actor(), Some(opponent));
        for card in [creature, ineligible] {
            let object = game.object_ref(card);
            reject(&mut game, Action::SelectResolutionExile { object });
        }
        let object = game.object_ref(enchantment);
        let action = Action::SelectResolutionExile { object };
        let saved = game.snapshot();
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let mut indexed = game.clone();
        indexed
            .apply_index(opponent, game.revision(), index)
            .unwrap();
        act(&mut game, action);
        assert_eq!(game.object_view(enchantment).zone, Zone::Exile);
        assert_eq!(game.object_view(other).zone, Zone::Battlefield);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(indexed.snapshot(), saved);
    }
}
