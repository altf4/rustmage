//! Target acquisition on a creature permanent retains the exact excluded object.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OpponentTargetsControlledCreature,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::OtherThanEventObject {
            base: &TargetRule::creature_you_control(),
            excluded: None,
        })),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn opponent_targeting_a_creature_spell_does_not_trigger() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let creature = put(&mut game, player, "Hill Giant", Zone::Hand);
    let counter = put(&mut game, player.other(), "Essence Scatter", Zone::Hand);
    cast(&mut game, creature, None);
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(observer, CounterKind::PlusOne), 0);
}

#[test]
fn opponent_target_excludes_the_exact_old_incarnation_and_allows_the_observer() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let creature = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, player.other(), "Giant Growth", Zone::Hand);
        act(&mut game, Action::Pass);
        cast(&mut game, spell, Some(Target::Card(creature)));

        let targeting = |id| Action::ChooseTarget {
            target: Target::Card(id),
        };
        assert_eq!(game.actor(), Some(player));
        reject(&mut game, targeting(creature));
        let mut returned = game.clone();
        act(&mut game, targeting(observer));
        settle(&mut game);
        assert_eq!(game.counter_count(observer, CounterKind::PlusOne), 1);
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 0);

        let old = returned.object_ref(creature);
        returned.move_object(creature, Zone::Exile);
        returned.move_object(creature, Zone::Battlefield);
        assert_ne!(returned.object_ref(creature), old);
        act(&mut returned, targeting(creature));
        settle(&mut returned);
        assert_eq!(returned.counter_count(creature, CounterKind::PlusOne), 1);
        assert_eq!(stats(&returned, creature), (4, 4));
    }
}
