//! A turn-relative attachment grant follows its grantor's controller.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const BONUS: Stats = Stats {
    power: 1,
    toughness: 1,
};
const BLOWGUN: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::AttachmentStats(BONUS)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AttachmentTurnChoiceKeywords {
                own_turn: Keywords::DEATHTOUCH,
                other_turn: Keywords::REACH,
            }],
        },
    ],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: BONUS,
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn assert_keywords(game: &Game, card: CardId, deathtouch: bool, reach: bool) {
    let keywords = game.keywords(card);
    assert_eq!(keywords.contains(Keywords::DEATHTOUCH), deathtouch);
    assert_eq!(keywords.contains(Keywords::REACH), reach);
}

#[test]
fn the_keywords_follow_grantor_control_and_switch_at_the_next_turn() {
    let bow = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(bow, &BLOWGUN)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let equipment = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let grantor = game.object_ref(equipment);
        let bearer = game.object_ref(first);
        let replacement_bearer = game.object_ref(second);
        assert_keywords(&game, first, false, false);
        assert!(game.attach_equipment(grantor, bearer));
        assert_keywords(&game, first, true, false);
        assert_eq!(stats(&game, first), (3, 3));

        game.record_control(bearer, other, None);
        assert_keywords(&game, first, true, false);
        game.record_control(grantor, other, None);
        assert_keywords(&game, first, false, true);

        game.state.step = Step::End;
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.state.active, other);
        assert_keywords(&game, first, true, false);

        game.remove_abilities_exact(&[bearer], grantor, other, AnimationDuration::Permanent);
        assert_keywords(&game, first, false, false);
        assert_eq!(stats(&game, first), (3, 3));
        assert!(game.attach_equipment(grantor, replacement_bearer));
        assert_keywords(&game, second, true, false);
        assert!(game.attach_equipment(grantor, bearer));
        assert_keywords(&game, first, true, false);
        assert_keywords(&game, second, false, false);

        game.remove_abilities_exact(&[grantor], bearer, other, AnimationDuration::Permanent);
        assert_keywords(&game, first, false, false);
        assert_eq!(stats(&game, first), (2, 2));
    }
}
