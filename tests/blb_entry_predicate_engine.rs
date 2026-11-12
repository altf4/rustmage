//! Individual entry predicates use the exact source and effective entrant.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SourceOr(
                &EntryPredicate::CreatureWithKeyword(Keywords::FLYING),
            )),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SourceOr(
                &EntryPredicate::CreatureToughnessGreaterThanPower,
            )),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
    ],
    ..CardRules::EMPTY
};

const ENTRANT: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING,
    ..CardRules::EMPTY
};

fn settle_triggers(game: &mut Game) {
    for _ in 0..32 {
        if game.state.pending == Pending::Priority && game.state.stack.is_empty() {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::PutTrigger { .. } | Action::Pass)
        });
    }
    panic!("entry trigger did not settle");
}

#[test]
fn entry_predicates_include_the_source_and_use_post_entry_keywords_and_stats() {
    let _rules =
        fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER), (Card::HillGiant, &ENTRANT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, source, None);
    settle_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);

    let excluded = put(&mut game, player, "Goblin Piker", Zone::Hand);
    cast(&mut game, excluded, None);
    settle_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);

    let both = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, both, None);
    settle_triggers(&mut game);
    assert!(game.object_view(both).keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, both), (2, 3));
    assert_eq!(game.state.players[player.index()].life, 26);
}
