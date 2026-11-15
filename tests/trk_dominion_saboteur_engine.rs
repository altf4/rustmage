//! As-entry copy counter snapshots are additional entry counters, not copiable values.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const COPY_WITH_COUNTERS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[
                BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                },
                BattlefieldFilter {
                    kinds: 16,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                },
            ],
            optional: true,
        }),
        entry_copy_options: Some(&EntryCopyOptions {
            source: EntryCopySource::BattlefieldWithCounters,
            exceptions: CopyExceptions {
                remove_legendary: true,
                ..CopyExceptions::NONE
            },
            ..EntryCopyOptions::DEFAULT
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn wait_copy(game: &mut Game) {
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::EntryCopy { .. }) {
            return;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        act(game, Action::Pass);
    }
    panic!("missing as-entry copy choice");
}

fn choose(game: &mut Game, object: Option<ObjectRef>) {
    act(game, Action::ChooseEntryCopy { object });
}

#[test]
fn copied_donor_counters_are_frozen_additional_and_noncopyable() {
    let source_card = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[(source_card, &COPY_WITH_COUNTERS)]);
    let mut game = scenario(Player::P0);
    let donor = put(
        &mut game,
        Player::P1,
        "Giada, Font of Hope",
        Zone::Battlefield,
    );
    game.set_counters(
        donor,
        Counters::of(CounterKind::PlusOne, 2)
            .with(CounterKind::Stun, 1)
            .with(CounterKind::Charge, 3),
    );
    let entrant = put(&mut game, Player::P0, "Bear Cub", Zone::Hand);
    cast(&mut game, entrant, None);
    wait_copy(&mut game);
    let donor_ref = game.object_ref(donor);
    let before = game.snapshot();
    let action = Action::ChooseEntryCopy {
        object: Some(donor_ref),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let revision = game.revision();
    let mut indexed = game.clone();
    indexed.apply_index(Player::P0, revision, index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    assert_eq!(indexed.snapshot(), before);
    choose(&mut indexed, None);
    settle(&mut indexed);
    assert_eq!(indexed.object_view(entrant).card, source_card);
    assert_eq!(indexed.counter_count(entrant, CounterKind::PlusOne), 0);

    settle(&mut game);
    let result = game.object_view(entrant);
    assert_eq!(result.card, Card::from_name("Giada, Font of Hope").unwrap());
    assert!(!result.is_legendary());
    for (kind, count) in [
        (CounterKind::PlusOne, 2),
        (CounterKind::Stun, 1),
        (CounterKind::Charge, 3),
    ] {
        assert_eq!(game.counter_count(entrant, kind), count);
    }
    let follower = put(&mut game, Player::P0, "Waxen Shapethief", Zone::Hand);
    cast(&mut game, follower, None);
    wait_copy(&mut game);
    let entrant_ref = game.object_ref(entrant);
    choose(&mut game, Some(entrant_ref));
    settle(&mut game);
    assert_eq!(game.object_view(follower).card, result.card);
    // The first copied Giada gives the second entering Angel one +1/+1 counter;
    // the donor's own two +1/+1, stun, and charge counters do not copy.
    assert_eq!(game.counter_count(follower, CounterKind::PlusOne), 1);
    for kind in [CounterKind::Stun, CounterKind::Charge] {
        assert_eq!(game.counter_count(follower, kind), 0);
    }
}

#[test]
fn artifact_donor_and_counter_doubler_apply_before_entry() {
    let source_card = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[(source_card, &COPY_WITH_COUNTERS)]);
    let mut game = scenario(Player::P0);
    let artifact = put(&mut game, Player::P1, "Short Bow", Zone::Battlefield);
    game.set_counters(artifact, Counters::of(CounterKind::Charge, 2));
    put(&mut game, Player::P0, "Doubling Season", Zone::Battlefield);
    let entrant = put(&mut game, Player::P0, "Bear Cub", Zone::Hand);
    cast(&mut game, entrant, None);
    wait_copy(&mut game);
    let artifact_ref = game.object_ref(artifact);
    choose(&mut game, Some(artifact_ref));
    settle(&mut game);
    assert_eq!(
        game.object_view(entrant).card,
        Card::from_name("Short Bow").unwrap()
    );
    assert_eq!(game.counter_count(entrant, CounterKind::Charge), 4);
}
