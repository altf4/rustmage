//! Event-relative target rules distinguish a live entrant from departure LKI.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const GRAVE_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const DEATH_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
            kinds: 1,
            keywords: Keywords::FLYING,
            relation: CastRelation::You,
            subject: DepartureSubject::SelfOrOtherMatching,
            destinations: &[Zone::Graveyard],
            ..DepartureRule::ANY
        }),
        targets: one(TargetRule::Filtered {
            base: &GRAVE_CREATURE,
            predicate: &TargetPredicate::ManaValueLessThanEvent { other: true },
        }),
        program: &[Instruction::ReturnToBattlefield],
    }],
    ..CardRules::EMPTY
};
const ENTRY_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureEnters {
            relation: CastRelation::You,
            other: false,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::creature_you_control(),
                predicate: &TargetPredicate::ManaValueLessThanEvent { other: false },
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ReturnToHand],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn event(game: &Game) -> &EventObjectView {
    game.state
        .announcing
        .as_ref()
        .unwrap()
        .item
        .event_object
        .as_ref()
        .unwrap()
}

#[test]
fn simultaneous_deaths_capture_each_copy_value_and_exclude_the_dying_card() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &DEATH_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let flying_copy = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let nonflying = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let opposing_flyer = put(&mut game, player.other(), "Serra Angel", Zone::Battlefield);
    let finality_flyer = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    game.add_counter(finality_flyer, CounterKind::Finality, 1);
    let copy = game.object_ref(flying_copy);
    game.apply_temporary_copy(
        copy,
        CopyRecipe::new(named("Serra Angel")),
        CopyDeadline::EndOfTurn,
    );
    let self_ref = game.object_ref(observer);
    let four = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let five = put(&mut game, player, "Serra Angel", Zone::Graveyard);
    let one = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
    let opposing_grave = put(&mut game, player.other(), "Healer's Hawk", Zone::Graveyard);
    let sweep = put(&mut game, player, "Day of Judgment", Zone::Hand);
    cast(&mut game, sweep, None);
    pass_pair(&mut game);

    assert_eq!(game.object_view(observer).zone, Zone::Graveyard);
    assert_eq!(game.object_view(nonflying).zone, Zone::Graveyard);
    assert_eq!(game.object_view(opposing_flyer).zone, Zone::Graveyard);
    assert_eq!(game.object_view(finality_flyer).zone, Zone::Exile);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    let first = game
        .state
        .ordering_triggers
        .iter()
        .find(|item| {
            item.event_object
                .as_ref()
                .is_some_and(|event| event.object == copy)
        })
        .unwrap()
        .id;
    act(&mut game, Action::PutTrigger { id: first });
    assert_eq!(event(&game).facts, EventObjectFacts::DepartureManaValue(5));
    reject(&mut game, target(flying_copy));
    reject(&mut game, target(five));
    reject(&mut game, target(opposing_grave));
    act(&mut game, target(four));

    assert_eq!(event(&game).object, self_ref);
    assert_eq!(event(&game).facts, EventObjectFacts::DepartureManaValue(4));
    reject(&mut game, target(four));
    act(&mut game, target(one));
    settle(&mut game);
    assert_eq!(game.object_view(one).zone, Zone::Battlefield);
    assert_eq!(game.object_view(four).zone, Zone::Battlefield);
    assert_eq!(game.object_view(flying_copy).zone, Zone::Graveyard);
}

#[test]
fn entry_comparison_uses_current_value_and_refreshes_lki_before_a_blink() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &ENTRY_OBSERVER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let smaller = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let equal = put(&mut game, player, "Serra Angel", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Llanowar Elves",
        Zone::Battlefield,
    );
    let entrant = put(&mut game, player, "Serra Angel", Zone::Hand);
    cast(&mut game, entrant, None);
    pass_pair(&mut game);
    let exact = game.object_ref(entrant);
    assert_eq!(event(&game).object, exact);
    reject(&mut game, target(equal));
    reject(&mut game, target(opposing));
    reject(&mut game, target(entrant));

    let choosing = game.snapshot();
    let action = target(smaller);
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|choice| *choice == action)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(indexed.snapshot(), game.snapshot());
    let chosen = game.snapshot();

    game.move_object(source, Zone::Exile);
    game.move_object(entrant, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(smaller).zone, Zone::Hand);

    game.restore(&chosen).unwrap();
    game.apply_temporary_copy(
        exact,
        CopyRecipe::new(Card::GrizzlyBears),
        CopyDeadline::EndOfTurn,
    );
    settle(&mut game);
    assert_eq!(game.object_view(smaller).zone, Zone::Battlefield);

    game.restore(&chosen).unwrap();
    game.apply_temporary_copy(
        exact,
        CopyRecipe::new(Card::GrizzlyBears),
        CopyDeadline::EndOfTurn,
    );
    game.move_object(source, Zone::Exile);
    game.move_object(entrant, Zone::Exile);
    game.move_object(entrant, Zone::Battlefield);
    assert_ne!(game.object_ref(entrant), exact);
    assert_eq!(
        game.state
            .stack
            .last()
            .unwrap()
            .event_object
            .as_ref()
            .unwrap()
            .facts,
        EventObjectFacts::BattlefieldManaValue(2)
    );
    settle(&mut game);
    assert_eq!(game.object_view(smaller).zone, Zone::Battlefield);

    game.restore(&choosing).unwrap();
    act(&mut game, Action::FinishTargets);
    settle(&mut game);
    assert_eq!(game.object_view(smaller).zone, Zone::Battlefield);
}

#[test]
fn source_entry_also_supplies_the_exact_value_for_an_optional_target() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &ENTRY_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let smaller = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let source = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    assert_eq!(event(&game).object, game.object_ref(source));
    assert_eq!(
        event(&game).facts,
        EventObjectFacts::BattlefieldManaValue(4)
    );
    act(&mut game, target(smaller));
    settle(&mut game);
    assert_eq!(game.object_view(smaller).zone, Zone::Hand);
}
