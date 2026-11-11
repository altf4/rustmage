//! Additive entry-copy subtypes remain copiable on subsequent copies.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ADDITIVE_COPY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        entry_copy_options: Some(&EntryCopyOptions {
            exceptions: CopyExceptions::SHAPESHIFTER_ROGUE,
            ..EntryCopyOptions::DEFAULT
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn enter_copy(game: &mut Game, card: CardId) {
    cast(game, card, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert!(matches!(game.state.pending, Pending::EntryCopy { .. }));
}

fn copy(object: ObjectRef) -> Action {
    Action::ChooseEntryCopy {
        object: Some(object),
    }
}

#[test]
fn added_subtypes_are_copied_with_printed_subtypes_and_without_counters() {
    let bear = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[(bear, &ADDITIVE_COPY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let donor = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    game.add_counter(donor, CounterKind::PlusOne, 2);
    let enemy = put(
        &mut game,
        player.other(),
        "Savannah Lions",
        Zone::Battlefield,
    );
    let entrant = put(&mut game, player, "Bear Cub", Zone::Hand);
    enter_copy(&mut game, entrant);
    let enemy = game.object_ref(enemy);
    reject(&mut game, copy(enemy));

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseEntryCopy { object: None });
    assert_eq!(declined.object_view(entrant).card, bear);
    assert_eq!(stats(&declined, entrant), (2, 2));

    let object = game.object_ref(donor);
    let action = copy(object);
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let follower = put(&mut game, player, "Waxen Shapethief", Zone::Hand);
    enter_copy(&mut game, follower);
    let object = game.object_ref(entrant);
    act(&mut game, copy(object));
    settle(&mut game);
    for card in [entrant, follower] {
        let view = game.object_view(card);
        assert_eq!(view.name, "Llanowar Elves");
        assert_eq!(stats(&game, card), (1, 1));
        for subtype in ["Elf", "Druid", "Shapeshifter", "Rogue"] {
            assert!(view.has_subtype(subtype), "missing {subtype}");
        }
        assert!(view.copy_recipe.unwrap().exceptions.shapeshifter_rogue);
    }
}

#[test]
fn repeated_battlefield_copy_replacements_remain_optional() {
    let bear = Card::from_name("Bear Cub").unwrap();
    let _rules = fixture_rules::activate(&[(bear, &ADDITIVE_COPY)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let donor = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let entrant = put(&mut game, player, "Bear Cub", Zone::Hand);
    enter_copy(&mut game, entrant);
    let object = game.object_ref(donor);
    for _ in 0..3 {
        act(&mut game, copy(object));
        assert!(matches!(game.state.pending, Pending::EntryCopy { .. }));
        assert_eq!(game.object_view(entrant).zone, Zone::Stack);
    }
    act(&mut game, Action::ChooseEntryCopy { object: None });
    assert_eq!(game.state.pending, Pending::Priority);
    let view = game.object_view(entrant);
    assert_eq!(view.card, bear);
    assert!(view.has_subtype("Bear"));
    assert!(view.has_subtype("Shapeshifter"));
    assert!(view.has_subtype("Rogue"));
}
