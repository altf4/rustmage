//! Attack trigger targets bind the defending player of the actual declaration.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ATTACKER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksDefendingPlayer,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::DefendingCreature)),
        program: &[Instruction::Damage(1)],
    }],
    ..CardRules::EMPTY
};
const COPY_TRIGGER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::TriggeredAbility {
            relation: 1,
        })),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};

fn select(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

#[test]
fn attack_at_a_planeswalker_and_copied_trigger_keep_the_original_defending_player() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ATTACKER), (opt, &COPY_TRIGGER)]);
    let attacker = Player::P1;
    let defender = attacker.other();
    let mut game = scenario(attacker);
    let source = put(&mut game, attacker, "Grizzly Bears", Zone::Battlefield);
    let allied = put(&mut game, attacker, "Hill Giant", Zone::Battlefield);
    let first = put(&mut game, defender, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, defender, "Hill Giant", Zone::Battlefield);
    let walker = put(
        &mut game,
        defender,
        "Oko, the Ringleader",
        Zone::Battlefield,
    );
    let copy = put(&mut game, defender, "Opt", Zone::Hand);
    game.add_counter(walker, CounterKind::Loyalty, 6);
    let planeswalker = game.object_ref(walker);
    game.begin_step(Step::BeginCombat);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::AttackPlaneswalker {
            card: source,
            defender: planeswalker,
        },
    );
    act(&mut game, Action::FinishAttackers);
    reject(&mut game, select(allied));
    act(&mut game, select(first));
    let trigger = game.state.stack.last().unwrap().id;
    game.move_object(walker, Zone::Graveyard);

    act(&mut game, Action::Pass);
    game.state.players[defender.index()].mana[1] = 1;
    cast(&mut game, copy, Some(Target::Stack(trigger)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let retarget = |card| Action::SetStackTarget {
        index: 0,
        target: Target::Card(card),
    };
    reject(&mut game, retarget(allied));
    act(&mut game, retarget(second));
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.object_view(first).damage, 1);
    assert_eq!(game.object_view(second).damage, 1);
    assert_eq!(game.object_view(allied).damage, 0);
}
