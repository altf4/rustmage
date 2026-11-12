//! Characteristic additions after an exact, successful entry.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: 3,
        })),
        &[
            Instruction::CaptureEntryResult,
            Instruction::ReturnToBattlefieldThenCounters(&[(CounterKind::Flying, 1)]),
            Instruction::AddEntryResultCharacteristics(CharacteristicAddition {
                card_types: 0,
                creature_types: &["Spirit"],
                colors: 0,
                duration: AnimationDuration::Permanent,
            }),
        ],
    ),
    ..CardRules::EMPTY
};

struct ResetPrevention;

impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    }
}

#[test]
fn add_a_type_to_the_successful_return_only_for_its_new_incarnation() {
    let spell = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &RETURN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let returning = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let bystander = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let old = game.object_ref(returning);
    let spell = spawn(&mut game, player, spell, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(returning)));
    let mut prevented = game.clone();
    settle(&mut game);
    let entered = game.object_view(returning);
    assert_ne!(entered.reference, old);
    assert_eq!(entered.zone, Zone::Battlefield);
    assert!(entered.has_subtype("Bear"));
    assert!(entered.has_subtype("Spirit"));
    assert!(entered.keywords.contains(Keywords::FLYING));
    assert_eq!(game.counter_count(returning, CounterKind::Flying), 1);
    assert!(!game.object_view(bystander).has_subtype("Spirit"));

    game.move_object(returning, Zone::Exile);
    game.move_object(returning, Zone::Battlefield);
    assert!(!game.object_view(returning).has_subtype("Spirit"));
    assert_eq!(game.counter_count(returning, CounterKind::Flying), 0);

    let _reset = ResetPrevention;
    prevented.state.zone_rules.get_or_insert_default();
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(old)));
    settle(&mut prevented);
    assert_eq!(prevented.object_ref(returning), old);
    assert_eq!(prevented.object_view(returning).zone, Zone::Graveyard);
    assert!(!prevented.object_view(returning).has_subtype("Spirit"));
    assert!(!prevented.object_view(bystander).has_subtype("Spirit"));
}
