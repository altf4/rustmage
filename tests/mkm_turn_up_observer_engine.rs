//! A single subtype-qualified observer watches entries and completed turn-ups.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DETECTIVES: &[&str] = &["Detective"];
const COUNTER_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersOrTurnsFaceUpYou {
            entry: EntryPredicate::SubtypeAny(DETECTIVES),
            other_on_entry: false,
            face_up_subtype: "Detective",
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersOnEventPermanent {
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const DETECTIVE: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    ..CardRules::EMPTY
};
const OTHER_ENTRY_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersOrTurnsFaceUpYou {
            entry: EntryPredicate::SubtypeAny(DETECTIVES),
            other_on_entry: true,
            face_up_subtype: "Detective",
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const SELF_OR_OTHER_ENTRY_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersOrTurnsFaceUpYou {
            entry: EntryPredicate::SourceOrSubtypeAny(DETECTIVES),
            other_on_entry: false,
            face_up_subtype: "Detective",
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn cloak(game: &mut Game, player: Player, card: Card) -> ObjectRef {
    let hidden = spawn(game, player, card, Zone::Library);
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());
    let object = game.object_ref(hidden);
    assert!(game.is_face_down(object));
    object
}

fn turn_up(game: &mut Game, object: ObjectRef) {
    choose(
        game,
        |action| matches!(action, Action::BeginFaceUp { object: candidate, .. } if *candidate == object),
    );
    choose(game, |action| matches!(action, Action::PayFaceUp { .. }));
    settle(game);
}

#[test]
fn entries_and_turn_ups_counter_the_exact_controlled_detective() {
    let observer = named("Authority of the Consuls");
    let detective = named("Novice Inspector");
    let _rules = fixture_rules::activate(&[(observer, &COUNTER_OBSERVER), (detective, &DETECTIVE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    spawn(&mut game, player, observer, Zone::Battlefield);
    let entrant = spawn(&mut game, player, detective, Zone::Hand);
    cast(&mut game, entrant, None);
    settle(&mut game);
    assert_eq!(game.counter_count(entrant, CounterKind::PlusOne), 1);

    let hidden = cloak(&mut game, player, detective);
    assert!(game.state.pending_triggers.is_empty());
    assert_eq!(game.counter_count(hidden.card, CounterKind::PlusOne), 0);
    turn_up(&mut game, hidden);
    assert_eq!(game.object_ref(hidden.card), hidden);
    assert_eq!(game.counter_count(hidden.card, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(entrant, CounterKind::PlusOne), 1);

    let non_detective = cloak(&mut game, player, named("Bear Cub"));
    turn_up(&mut game, non_detective);
    assert_eq!(
        game.counter_count(non_detective.card, CounterKind::PlusOne),
        0
    );

    let opposing = cloak(&mut game, opponent, detective);
    assert!(game.turn_manifest_face_up(opposing, player));
    assert!(game.state.pending_triggers.is_empty());
    assert_eq!(game.counter_count(opposing.card, CounterKind::PlusOne), 0);
}

#[test]
fn entry_can_exclude_or_include_the_source_independently_of_the_reveal() {
    let detective = named("Novice Inspector");
    let other = named("Grizzly Bears");
    let _rules = fixture_rules::activate(&[
        (detective, &OTHER_ENTRY_OBSERVER),
        (other, &SELF_OR_OTHER_ENTRY_OBSERVER),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let first = spawn(&mut game, player, detective, Zone::Hand);
    cast(&mut game, first, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    let second = spawn(&mut game, player, other, Zone::Hand);
    cast(&mut game, second, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    game.move_object(first, Zone::Exile);
    let hidden = cloak(&mut game, player, detective);
    game.move_object(second, Zone::Exile);
    turn_up(&mut game, hidden);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}
