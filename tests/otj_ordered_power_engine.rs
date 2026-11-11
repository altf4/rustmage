//! The second effect measures current creature powers after the first effect's move.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ORDERED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Nontoken,
            }),
            TargetSpec::one(TargetRule::Creature),
        ]),
        &[
            Instruction::BlinkExiledTargetsUntilEndStep,
            Instruction::SelectGroup(1),
            Instruction::DestroyTargetsUnlessGreaterCreaturePower,
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn checking_power_after_blink_allows_a_tie_and_preserves_the_exact_delayed_return() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &ORDERED)]);
    let player = Player::P0;
    let other = player.other();
    for blink in ["largest", "peer", "same"] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Shock", Zone::Hand);
        let largest = put(&mut game, other, "Hill Giant", Zone::Battlefield);
        let peer = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let victim = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let first = match blink {
            "largest" => largest,
            "peer" => peer,
            _ => victim,
        };
        let original = game.object_ref(first);
        game.record_control(original, player, None);
        announce(
            &mut game,
            source,
            &[Target::Card(first), Target::Card(victim)],
        );
        settle(&mut game);
        assert_eq!(game.object_view(first).zone, Zone::Exile);
        assert_eq!(
            game.object_view(victim).zone,
            match blink {
                "largest" => Zone::Graveyard,
                "peer" => Zone::Battlefield,
                _ => Zone::Exile,
            }
        );

        game.begin_step(Step::End);
        game.give_priority();
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(game.object_view(first).zone, Zone::Battlefield);
        assert_eq!(game.object_view(first).controller, other);
        assert_ne!(game.object_ref(first), original);
    }
}
