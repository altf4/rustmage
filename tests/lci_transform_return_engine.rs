//! Death-triggered transformed returns use the physical card's exact movement.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const CREATURE: &str = "Brigid, Clachan's Heart";
const LAND_FRONT: &str = "Sidequest: Catch a Fish";
const LAND_BACK: &str = "Cooking Campsite";

const DEATH_RETURN: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::SelfDies,
    targets: TargetPlan::NONE,
    program: &[Instruction::ReturnDepartedSourceTransformedTapped { counters: &[] }],
};

const RETURN: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[DEATH_RETURN],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const RETURN_WITH_TIME: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        program: &[Instruction::ReturnDepartedSourceTransformedTapped {
            counters: &[(CounterKind::Time, 3)],
        }],
        ..DEATH_RETURN
    }],
    ..RETURN
};

const CHOOSE_LAND_COLOR: CardRules = CardRules {
    effect: Effect::Land,
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::Color),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn destroy(game: &mut Game, card: CardId) {
    let player = game.actor().expect("actor before destruction spell");
    let spell = put(game, player, "Murder", Zone::Hand);
    cast(game, spell, Some(Target::Card(card)));
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn color(value: ChoiceColor) -> Action {
    Action::ChooseEntryValue {
        value: EntryChoiceValue::Color(value),
    }
}

#[test]
fn stolen_source_returns_tapped_and_transformed_to_its_owner() {
    let front = Card::from_name(CREATURE).unwrap();
    let _rules = fixture_rules::activate(&[(front, &RETURN)]);
    for owner in [Player::P0, Player::P1] {
        let mut game = scenario(owner);
        let card = put(&mut game, owner, CREATURE, Zone::Battlefield);
        let original = game.object_ref(card);
        game.record_control(original, owner.other(), None);
        game.add_counter(card, CounterKind::PlusOne, 1);

        destroy(&mut game, card);
        let dead = game.object_view(card);
        assert_eq!(dead.zone, Zone::Graveyard);
        assert_eq!(dead.card, front);
        assert_ne!(dead.reference, original);
        let trigger = game.state.stack.last().expect("return trigger");
        assert_eq!(trigger.controller, owner.other());
        assert!(trigger.targets.is_empty());

        settle(&mut game);
        let returned = game.object_view(card);
        assert_eq!(returned.zone, Zone::Battlefield);
        assert_eq!(returned.controller, owner);
        assert_eq!(returned.card, front.back_face().unwrap());
        assert_eq!(returned.reference.generation, dead.reference.generation + 1);
        assert!(returned.tapped);
        assert_eq!(returned.counters.get(CounterKind::PlusOne), 0);
    }
}

#[test]
fn replaced_death_and_a_later_graveyard_incarnation_cannot_return() {
    let front = Card::from_name(CREATURE).unwrap();
    let _rules = fixture_rules::activate(&[(front, &RETURN)]);
    for exile_instead in [false, true] {
        let owner = Player::P0;
        let mut game = scenario(owner);
        let card = put(&mut game, owner, CREATURE, Zone::Battlefield);
        if exile_instead {
            put(&mut game, owner.other(), "Rest in Peace", Zone::Battlefield);
        }
        destroy(&mut game, card);
        let expected_zone = if exile_instead {
            assert!(game.state.stack.is_empty());
            Zone::Exile
        } else {
            assert_eq!(game.state.stack.len(), 1);
            let dead = game.object_ref(card);
            game.move_object(card, Zone::Exile);
            game.move_object(card, Zone::Graveyard);
            assert_ne!(game.object_ref(card), dead);
            Zone::Graveyard
        };
        let expected = game.object_ref(card);
        settle(&mut game);
        let object = game.object_view(card);
        assert_eq!(object.zone, expected_zone);
        assert_eq!(object.reference, expected);
        assert_eq!(object.card, front);
    }
}

#[test]
fn copies_follow_the_physical_cards_face_structure() {
    let front = Card::from_name(CREATURE).unwrap();
    let _rules = fixture_rules::activate(&[(front, &RETURN)]);
    let mut game = scenario(Player::P0);
    let donor = put(&mut game, Player::P1, CREATURE, Zone::Battlefield);
    let recipe = game.copiable_recipe(game.object_ref(donor));
    for (name, returned_face) in [
        ("Grizzly Bears", None),
        ("Tony Stark", Some("The Invincible Iron Man")),
    ] {
        let card = put(&mut game, Player::P0, name, Zone::Battlefield);
        game.apply_temporary_copy(game.object_ref(card), recipe, CopyDeadline::EndOfTurn);
        assert_eq!(game.object_view(card).card, front);
        destroy(&mut game, card);
        settle(&mut game);
        let result = game.object_view(card);
        assert_eq!(
            result.card,
            Card::from_name(returned_face.unwrap_or(name)).unwrap()
        );
        assert_eq!(
            result.zone,
            if returned_face.is_some() {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(result.tapped, returned_face.is_some());
    }

    game.stage_copy_creation(Player::P0, recipe, 1);
    let token = game.state.copy_recipes.as_ref().unwrap()[0].object.card;
    assert!(game.object_view(token).is_token());
    destroy(&mut game, token);
    settle(&mut game);
    assert_eq!(game.object_view(token).zone, Zone::Outside);
    assert_eq!(game.object_view(donor).zone, Zone::Battlefield);
}

#[test]
fn transforming_copy_enters_as_its_own_land_with_owned_counter_entry() {
    let front = Card::from_name(CREATURE).unwrap();
    let land = Card::from_name(LAND_BACK).unwrap();
    let _rules = fixture_rules::activate(&[(front, &RETURN_WITH_TIME), (land, &CHOOSE_LAND_COLOR)]);
    let owner = Player::P1;
    let controller = owner.other();
    let mut game = scenario(controller);
    put(&mut game, owner, "Doubling Season", Zone::Battlefield);
    let donor = put(&mut game, owner, CREATURE, Zone::Battlefield);
    let card = put(&mut game, owner, LAND_FRONT, Zone::Battlefield);
    let original = game.object_ref(card);
    game.record_control(original, controller, None);
    let recipe = game.copiable_recipe(game.object_ref(donor));
    game.apply_temporary_copy(original, recipe, CopyDeadline::EndOfTurn);
    assert_eq!(game.object_view(card).card, front);

    destroy(&mut game, card);
    let departed = game.object_ref(card);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(owner));
    assert!(matches!(game.state.pending, Pending::EntryValue { .. }));
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    assert_eq!(game.object_ref(card), departed);

    let blue = color(ChoiceColor::Blue);
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| action == &blue)
        .expect("blue entry choice");
    assert_eq!(game.apply(controller, blue.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), saved);

    let mut raw = game.clone();
    let mut indexed = game.clone();
    drop(game);
    act(&mut raw, blue);
    indexed.apply_index(owner, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());

    let returned = raw.object_view(card);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.card, land);
    assert_eq!(returned.controller, owner);
    assert_ne!(returned.reference, departed);
    assert!(returned.tapped);
    assert_eq!(returned.counters.get(CounterKind::Time), 6);
    assert_eq!(
        returned.entry_choice,
        Some(EntryChoiceValue::Color(ChoiceColor::Blue))
    );
    assert!(returned.temporary_copy.is_none());

    raw.restore(&saved).unwrap();
    act(&mut raw, color(ChoiceColor::White));
    settle(&mut raw);
    assert_eq!(raw.object_view(card).counters.get(CounterKind::Time), 6);
    assert_eq!(
        raw.object_view(card).entry_choice,
        Some(EntryChoiceValue::Color(ChoiceColor::White))
    );
    assert_eq!(
        indexed.object_view(card).entry_choice,
        returned.entry_choice
    );
}
