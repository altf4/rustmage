//! Counters qualify against live colors and the exact controlled creature set.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const COUNTERS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PutPermanentCountersWithColors {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            colors: 0,
            kind: CounterKind::PlusOne,
            amount: AmountValue::Fixed(1),
        }],
    ),
    ..CardRules::EMPTY
};

const COLOR_PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Color,
        fragments: &[StaticAbilityFragment::ControlledNonlandAllColors],
    }],
    ..CardRules::EMPTY
};

#[test]
fn current_colors_exclude_colored_creatures_and_include_cloaks() {
    let spell = Card::from_name("Opt").unwrap();
    let leyline = Card::from_name("Doubling Season").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &COUNTERS), (leyline, &COLOR_PROVIDER)]);
    let player = Player::P0;
    let opponent = player.other();
    let thopter = Card::from_catalogue_id(5266);
    let mut game = scenario(player);
    game.create_tokens(player, thopter, 1, false);
    assert!(!game.advance_entries());
    let token = game.controlled_token_ids(player).next().unwrap();
    game.create_tokens(opponent, thopter, 1, false);
    assert!(!game.advance_entries());
    let hostile = game.controlled_token_ids(opponent).next().unwrap();
    let cloak = put(&mut game, player, "Grizzly Bears", Zone::Library);
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let source = spawn(&mut game, player, spell, Zone::Hand);
    cast(&mut game, source, None);

    let mut colored = game.clone();
    spawn(&mut colored, player, leyline, Zone::Battlefield);
    settle(&mut colored);
    settle(&mut game);
    for id in [token, cloak] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(colored.counter_count(id, CounterKind::PlusOne), 0);
    }
    for id in [hostile, creature, artifact] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 0);
    }
}
