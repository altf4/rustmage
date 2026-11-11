//! Exact source-linked type replacement, ability order and Treasure payments.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WHITE: Mana = [1, 0, 0, 0, 0, 0];

const fn optional_artifact_or_creature(relation: i8) -> TargetSpec {
    TargetSpec {
        rule: TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 17,
            relation,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        min: 0,
        max: 1,
    }
}

const CONVERTER: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Groups(&[
            optional_artifact_or_creature(0),
            optional_artifact_or_creature(1),
        ]),
        program: &[Instruction::MakeTargetsTreasureWhileSourceRemains],
    }],
    ..CardRules::EMPTY
};

fn definition() -> Card {
    Card::from_name("Healer's Hawk").unwrap()
}

fn enter(game: &mut Game, source: CardId) {
    cast(game, source, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn assert_treasure(game: &Game, card: CardId) {
    let view = game.object_view(card);
    assert_eq!(view.kinds, 16);
    assert!(view.has_subtype("Treasure"));
}

#[test]
fn both_optional_targets_are_revalidated_and_the_exact_source_anchors_the_duration() {
    let _rules = fixture_rules::activate(&[(definition(), &CONVERTER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    let ours = put(
        &mut game,
        player,
        "Dwynen, Gilt-Leaf Daen",
        Zone::Battlefield,
    );
    let theirs = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    let original = game.object_view(ours);
    game.add_counter(ours, CounterKind::PlusOne, 1);
    game.grant_keyword_effects(&[ours], original.reference, Keywords::HEXPROOF);
    enter(&mut game, source);
    assert_eq!(game.actor(), Some(player));
    reject(&mut game, target(source));
    reject(&mut game, target(theirs));

    let mut skipped = game.clone();
    act(&mut skipped, Action::FinishTargets);
    act(&mut skipped, Action::FinishTargets);
    settle(&mut skipped);
    assert!(!skipped.has_subtype(ours, "Treasure"));
    assert!(!skipped.has_subtype(theirs, "Treasure"));

    let saved = game.snapshot();
    let mut indexed = game.clone();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == target(ours))
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, target(ours));
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, target(ours));
    assert_eq!(game.snapshot(), indexed.snapshot());
    reject(&mut game, target(ours));
    act(&mut game, target(theirs));

    let mut invalidated = game.clone();
    let old_target = invalidated.object_ref(theirs);
    invalidated.move_object(theirs, Zone::Graveyard);
    invalidated.move_object(theirs, Zone::Battlefield);
    assert_ne!(invalidated.object_ref(theirs), old_target);
    settle(&mut invalidated);
    assert_treasure(&invalidated, ours);
    assert!(!invalidated.has_subtype(theirs, "Treasure"));

    let mut departed = game.clone();
    departed.move_object(source, Zone::Graveyard);
    settle(&mut departed);
    assert!(!departed.has_subtype(ours, "Treasure"));
    assert!(!departed.has_subtype(theirs, "Treasure"));

    settle(&mut game);
    for card in [ours, theirs] {
        assert_treasure(&game, card);
        assert!(!game.has_subtype(card, "Elf"));
    }
    let view = game.object_view(ours);
    assert_eq!(view.reference, original.reference);
    assert_eq!(view.name, original.name);
    assert_eq!(view.colors, original.colors);
    assert!(view.is_legendary());
    assert!(!view.keywords.contains(Keywords::REACH));
    assert!(!view.keywords.contains(Keywords::HEXPROOF));
    assert_eq!(game.counter_count(ours, CounterKind::PlusOne), 1);

    let exact_source = game.object_ref(source);
    game.change_controller(exact_source, opponent);
    game.grant_keyword_effects(&[ours], exact_source, Keywords::LIFELINK);
    assert!(game.object_view(ours).keywords.contains(Keywords::LIFELINK));
    assert_treasure(&game, ours);
    assert_treasure(&game, theirs);
    act(&mut game, Action::Pass);
    reject(&mut game, Action::TapMana { card: theirs });

    game.move_object(source, Zone::Graveyard);
    assert!(game.object_view(ours).keywords.contains(Keywords::REACH));
    assert!(game.object_view(ours).keywords.contains(Keywords::HEXPROOF));
    assert_eq!(game.object_view(ours).kinds, 1);
    assert!(game.has_subtype(ours, "Elf"));
    assert!(!game.has_subtype(ours, "Treasure"));
    assert!(!game.has_subtype(theirs, "Treasure"));
    act(&mut game, Action::TapMana { card: theirs });
}

#[test]
fn a_converted_nonproducer_funds_a_spell_and_sacrificial_mana_remembers_treasure() {
    let _rules = fixture_rules::activate(&[(definition(), &CONVERTER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    let ours = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let theirs = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Savannah Lions", Zone::Hand);
    game.state.objects[ours as usize].summoning_sick = true;
    enter(&mut game, source);

    let mut enemy_only = game.clone();
    act(&mut enemy_only, Action::FinishTargets);
    act(&mut enemy_only, target(theirs));
    settle(&mut enemy_only);
    assert!(!enemy_only.has_subtype(ours, "Treasure"));
    assert_treasure(&enemy_only, theirs);
    assert!(!enemy_only.has_subtype(theirs, "Equipment"));

    act(&mut game, target(ours));
    act(&mut game, Action::FinishTargets);
    settle(&mut game);
    assert_treasure(&game, ours);
    assert!(game.has_subtype(theirs, "Equipment"));
    assert!(!game.has_subtype(theirs, "Treasure"));

    game.state.players[player.index()].mana = [0; 6];
    let exact = game.object_ref(ours);
    let tap = |production| Action::TapGrantedMana {
        card: ours,
        grantor: game.object_ref(source),
        production,
    };
    let deferred_mana = tap([0; 6]);
    let legal = game.legal_actions();
    for index in 0..6 {
        let mut mana = [0; 6];
        mana[index] = 1;
        assert_eq!(legal.contains(&tap(mana)), index < 5);
    }
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, deferred_mana);
    assert!(matches!(game.state.pending, Pending::EffectMana { .. }));
    assert_eq!(game.object_view(ours).zone, Zone::Graveyard);

    let saved = game.snapshot();
    let action = Action::ChooseMana { production: WHITE };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, action.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    let lots = game.observe(player).mana_lots;
    assert_eq!(lots.len(), 1);
    assert_eq!(game.mana_lots(player)[0].quality.producer, Some(exact));
    assert_eq!(lots[0].amounts, WHITE);
    assert!(lots[0].treasure_source);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}
