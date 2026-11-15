//! Promotion is an ordinary named counter: it survives owned branches and
//! participates in typed placement and removal without changing combat stats.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROMOTE: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::Promotion,
            amount: 3,
        }],
    ),
    ..CardRules::EMPTY
};
const DEMOTE: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::RemoveCounter {
            subject: Subject::Targets,
            kind: CounterKind::Promotion,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn promotion_counter_uses_named_storage_and_restores_exactly() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Giant Growth").unwrap(), &PROMOTE),
        (Card::from_name("Unsummon").unwrap(), &DEMOTE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let promote = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, promote, Some(Target::Card(creature)));
        settle(&mut game);
        assert_eq!(game.counter_count(creature, CounterKind::Promotion), 3);
        assert_eq!(game.object_view(creature).counters.get(CounterKind::Promotion), 3);
        assert_eq!(game.object_view(creature).power, 2);
        assert_eq!(game.object_view(creature).toughness, 2);

        let checkpoint = game.snapshot();
        let mut branch = game.clone();
        let demote = put(&mut branch, player, "Unsummon", Zone::Hand);
        cast(&mut branch, demote, Some(Target::Card(creature)));
        settle(&mut branch);
        assert_eq!(branch.counter_count(creature, CounterKind::Promotion), 2);
        assert_eq!(game.counter_count(creature, CounterKind::Promotion), 3);
        branch.restore(&checkpoint).unwrap();
        assert_eq!(branch.snapshot(), checkpoint);
    }
}
