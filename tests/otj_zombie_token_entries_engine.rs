//! Historical exact subtype entries and copies of one opposing token-entry batch.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const HISTORY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SubtypeAny(&["Zombie"])),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Entering,
            kind: CounterKind::PlusOne,
            amount: AmountValue::OtherZombieEntriesThisTurn,
        }],
    }],
    ..CardRules::EMPTY
};

const OPPONENT_BATCH: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::TokensEnterOpponent),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTappedCopiesOfEntryBatch],
        },
        TriggeredAbility {
            condition: TriggerCondition::TokensEnterYou,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLife(PlayerSet::Opponent, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};

const ZOMBIE: Card = Card::from_catalogue_id(5290);
const TREASURE: Card = Card::from_catalogue_id(5302);
const SPIRIT: Card = Card::from_catalogue_id(5326);

fn resolve_waiting(game: &mut Game) {
    game.prepare_triggers();
    for _ in 0..36 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
            && matches!(game.state.pending, Pending::Priority)
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("entry receipt scenarios exceeded their action budget");
}

#[test]
fn zombie_entry_history_keeps_departed_incarnations_and_simultaneous_members() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &HISTORY)]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let zombie = put(&mut game, player, "Diregraf Ghoul", Zone::Hand);
    let first_entry = game.move_object(zombie, Zone::Battlefield).after;
    game.move_object(zombie, Zone::Graveyard);

    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second_entry = game.move_object(zombie, Zone::Battlefield).after;
    assert_ne!(first_entry, second_entry);
    let twins = game.create_token_batch_capture(player, ZOMBIE, 2, false, None, true);
    game.create_token_batch_capture(other, ZOMBIE, 1, false, None, true);
    assert_eq!(game.state.pending_triggers.len(), 3);
    game.record_control(second_entry, other, None);
    resolve_waiting(&mut game);
    for object in std::iter::once(second_entry).chain(twins) {
        assert_eq!(game.object_view(object.card).counters.plus_one, 3);
    }
    assert_eq!(game.object_view(zombie).controller, other);
}

#[test]
fn opposing_batch_copies_distinct_recipes_tapped_once_including_a_departed_token() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OPPONENT_BATCH)]);
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let first = game.create_token_groups(
        other,
        &[
            TokenGroupView {
                token: TREASURE,
                amount: 1,
                tapped: false,
            },
            TokenGroupView {
                token: SPIRIT,
                amount: 1,
                tapped: false,
            },
        ],
        None,
        true,
    );
    assert_eq!(game.state.pending_triggers.len(), 1);
    let packet = game.state.pending_triggers.last().unwrap();
    let event = packet.event_object.as_ref().unwrap();
    assert!(matches!(&event.facts, EventObjectFacts::CopiableGroup(rows) if rows.len() == 2));
    let spirit = first
        .iter()
        .find(|object| game.object_view(object.card).card == SPIRIT)
        .unwrap();
    game.move_object(spirit.card, Zone::Graveyard);
    game.create_token_batch_capture(other, TREASURE, 1, false, None, true);
    assert_eq!(game.state.pending_triggers.len(), 1);

    resolve_waiting(&mut game);
    let created: Vec<_> = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.controller == player && object.zone == Zone::Battlefield && object.is_token()
        })
        .collect();
    assert_eq!(created.len(), 2);
    assert!(created.iter().all(|object| object.tapped));
    assert!(created.iter().any(|object| object.card == TREASURE));
    assert!(created.iter().any(|object| object.card == SPIRIT));
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[other.index()].life, 19);

    game.create_token_batch_capture(player, TREASURE, 2, false, None, true);
    resolve_waiting(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.state.players[other.index()].life, 18);
}
