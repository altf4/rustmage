//! Additive damage replacements qualify the actual event and source.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const TYPAL: ReplacementSpec = ReplacementSpec::ControlledSubtypeDamagePlus {
    subtypes: &["Lizard", "Mouse", "Otter", "Raccoon"],
    amount: 1,
};

const CLASS: ReplacementSpec = ReplacementSpec::ConditionalNoncombatDamageToOpponentPlus {
    condition: StatValue::SourceClassLevelAtLeast(3),
    amount: 2,
};

const TYPAL_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, TYPAL)],
}];

const CLASS_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, CLASS)],
}];

const TYPAL_PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: TYPAL_GROUPS,
    extras: &CardExtras {
        static_ability_groups: TYPAL_GROUPS,
        replacement_effects: &[TYPAL],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const CLASS_PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: CLASS_GROUPS,
    extras: &CardExtras {
        static_ability_groups: CLASS_GROUPS,
        replacement_effects: &[CLASS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn finish_damage_replacements(game: &mut Game) {
    for _ in 0..8 {
        if !game.advance_replacements(false) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::ChooseDamageEvent { .. })
        });
    }
    panic!("damage replacements did not settle");
}

#[test]
fn subtype_damage_increases_once_per_recipient_even_from_a_departed_changeling() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TYPAL_PROVIDER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let recipient = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
        let source = game.damage_source(changeling);
        game.move_object(changeling, Zone::Exile);
        game.apply_damage_batch(&[
            DamageEvent {
                preventable: true,
                source,
                target: Target::Card(recipient),
                amount: 1,
                combat: false,
            },
            DamageEvent {
                preventable: true,
                source,
                target: Target::Player(player.other()),
                amount: 1,
                combat: false,
            },
        ]);
        finish_damage_replacements(&mut game);
        assert_eq!(game.object_view(recipient).damage, 2);
        assert_eq!(game.state.players[player.other().index()].life, 18);
    }
}

#[test]
fn class_damage_increase_requires_level_three_noncombat_and_an_opposing_recipient() {
    let class = Card::from_name("Doubling Season").unwrap();
    let _rules = fixture_rules::activate(&[(class, &CLASS_PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = spawn(&mut game, player, class, Zone::Battlefield);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let damage = DamageEvent {
        preventable: true,
        source: game.damage_source(source),
        target: Target::Player(player.other()),
        amount: 1,
        combat: false,
    };
    game.state.objects[provider as usize].class_level = 2;
    game.apply_damage_batch(&[damage]);
    assert_eq!(game.state.players[player.other().index()].life, 19);

    game.state.objects[provider as usize].class_level = 3;
    game.apply_damage_batch(&[
        damage,
        DamageEvent {
            combat: true,
            ..damage
        },
        DamageEvent {
            target: Target::Player(player),
            ..damage
        },
    ]);
    finish_damage_replacements(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 15);
    assert_eq!(game.state.players[player.index()].life, 19);
}
