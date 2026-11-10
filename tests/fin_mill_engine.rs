mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ADDITIONAL_MILL: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::OpponentMillAdditional(4)],
    }],
    ..CardRules::EMPTY
};

const MILL_EACH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Mill(PlayerSet::Each, MillAmount::Fixed(1))],
    ),
    ..CardRules::EMPTY
};

const MILL_HALF_DOWN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Mill(
            PlayerSet::Each,
            MillAmount::HalfLibraryRoundedDown,
        )],
    ),
    ..CardRules::EMPTY
};

const MILL_THEN_REWARD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::MillThenReflexive {
            amount: 4,
            ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};

fn cast_mill(game: &mut Game, player: Player, name: &str) {
    let spell = put(game, player, name, Zone::Hand);
    cast(game, spell, None);
    settle(game);
}

#[test]
fn additive_mill_uses_the_recipient_and_caps_at_the_actual_library() {
    let _rules = fixture_rules::activate(&[
        (
            Card::from_name("Swiftfoot Boots").unwrap(),
            &ADDITIONAL_MILL,
        ),
        (Card::from_name("Opt").unwrap(), &MILL_EACH),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        for _ in 0..2 {
            put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        }
        let opponent_top = put(&mut game, player.other(), "Island", Zone::Library);
        let own_top = put(&mut game, player, "Island", Zone::Library);
        cast_mill(&mut game, player, "Opt");
        assert_eq!(game.state.players[player.index()].library.len(), 8);
        assert!(
            game.state.players[player.other().index()]
                .library
                .is_empty()
        );
        assert_eq!(game.object_view(opponent_top).zone, Zone::Graveyard);
        assert_eq!(game.object_view(own_top).zone, Zone::Graveyard);
        assert!(game.outcome().is_none());
    }
}

#[test]
fn half_mill_rounds_down_for_each_player_and_zero_stays_zero() {
    let _rules = fixture_rules::activate(&[
        (
            Card::from_name("Swiftfoot Boots").unwrap(),
            &ADDITIONAL_MILL,
        ),
        (Card::from_name("Opt").unwrap(), &MILL_HALF_DOWN),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    for _ in 0..7 {
        let card = *game.state.players[player.other().index()]
            .library
            .last()
            .unwrap();
        game.move_object(card, Zone::Graveyard);
    }
    put(&mut game, player, "Island", Zone::Library);
    cast_mill(&mut game, player, "Opt");
    assert_eq!(game.state.players[player.index()].library.len(), 5);
    assert_eq!(game.state.players[player.other().index()].library.len(), 1);
}

#[test]
fn replaced_full_mill_keeps_the_reflexive_event() {
    let _rules = fixture_rules::activate(&[
        (
            Card::from_name("Swiftfoot Boots").unwrap(),
            &ADDITIONAL_MILL,
        ),
        (Card::from_name("Opt").unwrap(), &MILL_THEN_REWARD),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let life_before = game.state.players[player.index()].life;
    cast_mill(&mut game, player, "Opt");
    assert!(game.state.players[player.index()].library.is_empty());
    assert_eq!(game.state.players[player.index()].life, life_before + 3);
}
