//! A controller's creature-only counter replacement and live counter-qualified hexproof.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::replacements::{CounterPlacement, ResumeMode};
use rustmage_core::{cards::*, engine::*};

const DOUBLE: ReplacementSpec =
    ReplacementSpec::ControlledCreatureCountersPlacedByController { factor: 2 };
const DOCTOR: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::ControlledOtherCounteredCreatureKeywords(Keywords::HEXPROOF),
            ],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
        },
    ],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn place(game: &mut Game, card: CardId, by: Player, amount: Counters) {
    let view = game.object_view(card);
    game.stage_counter_placements(
        vec![CounterPlacement {
            object: view.reference,
            placed_by: by,
            card: view.card,
            owner: view.owner,
            controller: view.controller,
            protector: view.battle_protector,
            subtypes: view.granted_subtypes,
            kind: view.kinds,
            power: view.power,
            amount,
            effect: true,
            entry_index: None,
            private: false,
        }],
        ResumeMode::Pending,
    );
    game.advance_replacements(true);
}

#[test]
fn counter_kinds_double_only_for_own_creatures_and_other_countered_creatures_gain_hexproof() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &DOCTOR)]);
    let mut game = scenario(Player::P0);
    let doctor = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let own = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, Player::P0, "Short Bow", Zone::Battlefield);
    assert!(!game.keywords(own).contains(Keywords::HEXPROOF));

    let mut mixed = Counters::of(CounterKind::PlusOne, 1);
    mixed.add(CounterKind::Stun, 1);
    place(&mut game, own, Player::P0, mixed);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(own, CounterKind::Stun), 2);
    assert!(game.keywords(own).contains(Keywords::HEXPROOF));

    place(
        &mut game,
        doctor,
        Player::P0,
        Counters::of(CounterKind::PlusOne, 1),
    );
    assert_eq!(game.counter_count(doctor, CounterKind::PlusOne), 2);
    assert!(!game.keywords(doctor).contains(Keywords::HEXPROOF));
    place(
        &mut game,
        opposing,
        Player::P0,
        Counters::of(CounterKind::PlusOne, 1),
    );
    place(
        &mut game,
        artifact,
        Player::P0,
        Counters::of(CounterKind::PlusOne, 1),
    );
    assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(artifact, CounterKind::PlusOne), 1);
    place(
        &mut game,
        own,
        Player::P1,
        Counters::of(CounterKind::Flying, 1),
    );
    assert_eq!(game.counter_count(own, CounterKind::Flying), 1);

    game.move_object(doctor, Zone::Graveyard);
    assert!(!game.keywords(own).contains(Keywords::HEXPROOF));
}

#[test]
fn copied_doctor_supplies_both_rules_and_loses_them_when_copy_ends() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &DOCTOR)]);
    let mut game = scenario(Player::P0);
    let copier = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let target = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    place(
        &mut game,
        target,
        Player::P0,
        Counters::of(CounterKind::PlusOne, 1),
    );
    assert_eq!(game.counter_count(target, CounterKind::PlusOne), 1);
    assert!(!game.keywords(target).contains(Keywords::HEXPROOF));

    let reference = game.object_ref(copier);
    game.set_copy_recipe(reference, CopyRecipe::new(Card::GrizzlyBears));
    let copied = game.snapshot();
    assert!(game.keywords(target).contains(Keywords::HEXPROOF));
    place(
        &mut game,
        target,
        Player::P0,
        Counters::of(CounterKind::Stun, 1),
    );
    assert_eq!(game.counter_count(target, CounterKind::Stun), 2);

    game.set_copy_recipe(reference, CopyRecipe::new(Card::Forest));
    assert!(!game.keywords(target).contains(Keywords::HEXPROOF));
    place(
        &mut game,
        target,
        Player::P0,
        Counters::of(CounterKind::Flying, 1),
    );
    assert_eq!(game.counter_count(target, CounterKind::Flying), 1);
    game.clear_copy_recipe(reference);
    assert!(!game.keywords(target).contains(Keywords::HEXPROOF));
    game.restore(&copied).unwrap();
    assert!(game.keywords(target).contains(Keywords::HEXPROOF));
    assert_eq!(game.counter_count(target, CounterKind::Stun), 0);
}
