//! Exact departure counters and non-targeted return choices across graveyards.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EITHER_GRAVEYARD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ChooseGraveyardReturnFromEither],
    ),
    ..CardRules::EMPTY
};

const OWN_GRAVEYARD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ChooseGraveyardReturnCount(AmountValue::Fixed(
            1,
        ))],
    ),
    ..CardRules::EMPTY
};

const COUNTER_DEPARTURE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
            kinds: 1,
            relation: CastRelation::You,
            subject: DepartureSubject::SelfOrOtherMatching,
            destinations: &[Zone::Graveyard],
            counters: DepartureCounters::Any,
            ..DepartureRule::ANY
        }),
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::PutEventCountersOnTargets],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn mandatory_untargeted_return_chooses_either_graveyard_and_stages_the_new_controller() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &EITHER_GRAVEYARD),
        (Card::Divination, &OWN_GRAVEYARD),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Impact Tremors", Zone::Battlefield);
    let yours = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let theirs = put(&mut game, opponent, "Hill Giant", Zone::Graveyard);
    let noncreature = put(&mut game, opponent, "Shock", Zone::Graveyard);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let own_spell = put(&mut game, player, "Divination", Zone::Hand);
    let select = |card| Action::SelectGraveyardReturn {
        object: game.object_ref(card),
    };
    let own_return = select(yours);
    let opposing_return = select(theirs);
    let invalid = select(noncreature);

    let mut own_only = game.clone();
    cast(&mut own_only, own_spell, None);
    pass_pair(&mut own_only);
    reject(&mut own_only, opposing_return.clone());
    act(&mut own_only, own_return.clone());
    settle(&mut own_only);
    assert_eq!(own_only.object_view(yours).zone, Zone::Battlefield);

    let mut empty = game.clone();
    for card in [yours, theirs] {
        empty.move_object(card, Zone::Exile);
    }
    cast(&mut empty, spell, None);
    settle(&mut empty);
    assert_eq!(empty.object_view(spell).zone, Zone::Graveyard);

    cast(&mut game, spell, None);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    assert!(game.observe(opponent).actions.is_empty());
    reject(&mut game, invalid);
    reject(&mut game, Action::FinishGraveyardReturn);

    let checkpoint = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == opposing_return)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, opposing_return);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), checkpoint);
    act(&mut indexed, own_return);
    settle(&mut indexed);
    assert_eq!(indexed.object_view(theirs).zone, Zone::Graveyard);

    settle(&mut game);
    let creature = game.object_view(theirs);
    assert_eq!(creature.zone, Zone::Battlefield);
    assert_eq!(creature.owner, opponent);
    assert_eq!(creature.controller, player);
    assert_eq!(game.object_view(yours).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 19);
}

#[test]
fn exact_departing_source_uses_the_shared_counter_gate_even_when_noncreature() {
    let fixture = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(fixture, &COUNTER_DEPARTURE)]);
    let player = Player::P1;
    let mut base = scenario(player);
    let source = spawn(&mut base, player, fixture, Zone::Battlefield);
    let receiver = put(&mut base, player, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut base, player, "Disenchant", Zone::Hand);
    assert!(!base.object_view(source).is_creature());
    let mut counterless = base.clone();
    cast(&mut counterless, spell, Some(Target::Card(source)));
    pass_pair(&mut counterless);
    assert_eq!(counterless.object_view(source).zone, Zone::Graveyard);
    assert!(counterless.state.stack.is_empty());
    assert_eq!(counterless.state.pending, Pending::Priority);

    base.add_counter(source, CounterKind::PlusOne, 2);
    base.add_counter(source, CounterKind::Flying, 1);
    cast(&mut base, spell, Some(Target::Card(source)));
    pass_pair(&mut base);
    act(
        &mut base,
        Action::ChooseTarget {
            target: Target::Card(receiver),
        },
    );
    settle(&mut base);
    assert_eq!(base.object_view(source).zone, Zone::Graveyard);
    assert_eq!(base.counter_count(receiver, CounterKind::PlusOne), 2);
    assert_eq!(base.counter_count(receiver, CounterKind::Flying), 1);
}
