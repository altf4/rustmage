//! Exact death counters use the simultaneous pre-event characteristics.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GAIN_ONE: &[Instruction] = &[Instruction::GainLife(PlayerSet::Controller, 1)];
const PLUS_ONE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesWithCounter {
            relation: CastRelation::You,
            kind: CounterKind::PlusOne,
        },
        targets: TargetPlan::NONE,
        program: GAIN_ONE,
    }],
    ..CardRules::EMPTY
};
const ANY: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesWithCounters {
            relation: CastRelation::You,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 10)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn the_exact_counter_observer_queues_once_per_qualifying_death_in_a_batch() {
    let exact_card = Card::from_name("Short Bow").unwrap();
    let any_card = Card::from_name("Basilisk Collar").unwrap();
    let _rules = fixture_rules::activate(&[(exact_card, &PLUS_ONE), (any_card, &ANY)]);
    let p = Player::P1;
    let mut game = scenario(p);
    put(&mut game, p, "Short Bow", Zone::Battlefield);
    put(&mut game, p, "Basilisk Collar", Zone::Battlefield);
    let one = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let stun = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let token = game.state.objects.len() as CardId;
    game.create_tokens(p, Card::test_index(5323), 1, false);
    for (card, kind, amount) in [
        (one, CounterKind::PlusOne, 2),
        (one, CounterKind::Stun, 1),
        (token, CounterKind::PlusOne, 1),
        (stun, CounterKind::Stun, 1),
        (foreign, CounterKind::PlusOne, 1),
    ] {
        game.add_counter(card, kind, amount);
    }
    let spell = put(&mut game, p, "Day of Judgment", Zone::Hand);
    game.state.players[p.index()].mana = [2, 0, 0, 0, 0, 2];
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.ordering_triggers.len(), 5);
    for _ in 0..24 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.pending == Pending::Priority
        {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.pending, Pending::Priority);
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE + 32);
    assert_eq!(game.state.players[p.other().index()].life, STARTING_LIFE);
}
