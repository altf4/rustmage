//! Attachment timestamp removal and live Class keyword grants.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SWORD: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::RemoveAttachedKeywords(Keywords::FLYING),
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 3,
                toughness: 3,
            }),
        ],
    }],
    ..CardRules::EMPTY
};

const FLYING_GRANT: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
            kinds: 1,
            other: false,
            keywords: Keywords::FLYING,
        }],
    }],
    ..CardRules::EMPTY
};

const CLASS: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ConditionalEquippedCreatureKeywords {
            condition: StatValue::AtLeast {
                value: &StatValue::Sum(&[
                    StatValue::ControllerTurn,
                    StatValue::SourceClassLevelAtLeast(3),
                ]),
                minimum: 2,
            },
            keywords: Keywords::DOUBLE_STRIKE.union(Keywords::HASTE),
        }],
    }],
    ..CardRules::EMPTY
};

fn attach(game: &mut Game, equipment: CardId, creature: CardId) {
    assert!(game.attach_equipment(game.object_ref(equipment), game.object_ref(creature)));
}

#[test]
fn attached_removal_keeps_other_keywords_and_uses_attachment_timestamp() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &SWORD),
        (Card::HillGiant, &FLYING_GRANT),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let wearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let spare = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let old_grant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    attach(&mut game, boots, wearer);
    let sword = put(&mut game, player, "Short Bow", Zone::Battlefield);
    assert!(game.keywords(wearer).contains(Keywords::FLYING));

    attach(&mut game, sword, wearer);
    assert!(!game.keywords(wearer).contains(Keywords::FLYING));
    assert!(game.keywords(wearer).contains(Keywords::HASTE));
    assert_eq!(stats(&game, wearer), (5, 5));

    let later_grant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    assert!(game.keywords(wearer).contains(Keywords::FLYING));
    attach(&mut game, sword, spare);
    attach(&mut game, sword, wearer);
    assert!(!game.keywords(wearer).contains(Keywords::FLYING));
    assert!(game.keywords(spare).contains(Keywords::FLYING));

    assert!(game.unattach(game.object_ref(sword)));
    game.move_object(later_grant, Zone::Graveyard);
    assert!(game.keywords(wearer).contains(Keywords::FLYING));
    assert_eq!(stats(&game, wearer), (2, 2));
    assert!(game.on_battlefield(old_grant));
}

#[test]
fn equipped_grant_checks_live_level_and_turn_of_the_creatures_controller() {
    let class_card = Card::from_name("Hunter's Talent").unwrap();
    let _rules = fixture_rules::activate(&[(class_card, &CLASS)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let class = put(&mut game, player, "Hunter's Talent", Zone::Battlefield);
        let wearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let plain = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let opponent_wearer = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let equipment = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
        attach(&mut game, equipment, wearer);
        let opposing_equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
        attach(&mut game, opposing_equipment, opponent_wearer);
        game.state.objects[class as usize].class_level = 2;
        let bonuses = Keywords::DOUBLE_STRIKE.union(Keywords::HASTE);
        assert!(!game.keywords(wearer).contains(bonuses));
        game.state.objects[class as usize].class_level = 3;
        assert!(game.keywords(wearer).contains(bonuses));
        assert!(!game.keywords(plain).contains(Keywords::DOUBLE_STRIKE));
        assert!(
            !game
                .keywords(opponent_wearer)
                .contains(Keywords::DOUBLE_STRIKE)
        );
        game.state.active = player.other();
        assert!(!game.keywords(wearer).contains(Keywords::DOUBLE_STRIKE));
    }
}
