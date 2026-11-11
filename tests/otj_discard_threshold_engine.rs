//! A private mandatory discard continues with its actual result and pre-move mana value.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DISCARD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: true,
            },
            min: 0,
            max: 1,
        }),
        &[Instruction::DiscardOneEachDrawUnlessManaValue {
            players: PlayerSet::Targeted,
            minimum: 4,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn only_a_targeted_player_who_did_not_discard_four_or_more_yields_a_draw() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &DISCARD)]);
    let player = Player::P0;
    let other = player.other();
    for (name, chosen, expected_zone, draw) in [
        (None, true, None, true),
        (Some("Shock"), false, Some(Zone::Hand), false),
        (Some("Shock"), true, Some(Zone::Exile), true),
        (Some("Hill Giant"), true, Some(Zone::Exile), false),
        (
            Some("Wilt-Leaf Liege"),
            true,
            Some(Zone::Battlefield),
            false,
        ),
    ] {
        let mut game = scenario(player);
        if name != Some("Wilt-Leaf Liege") {
            put(&mut game, player, "Rest in Peace", Zone::Battlefield);
        }
        let card = name.map(|name| put(&mut game, other, name, Zone::Hand));
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        let top = game.top_library_card(player).unwrap();
        cast(&mut game, spell, chosen.then_some(Target::Player(other)));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        if chosen && let Some(card) = card {
            let discard = Action::Discard { card };
            assert_eq!(game.actor(), Some(other));
            let private = game.observe(player);
            assert!(!private.actions.contains(&discard));
            assert!(!private.objects.iter().any(|object| object.id == card));
            act(&mut game, discard);
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(top).zone,
            if draw { Zone::Hand } else { Zone::Library }
        );
        if let Some(card) = card {
            assert_eq!(game.object_view(card).zone, expected_zone.unwrap());
        }
    }
}
