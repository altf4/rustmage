//! Exact trigger multiplicity follows actual sources and matched entry events.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const END_TRIGGER: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::AtStep {
        step: TriggerStep::End,
        relation: CastRelation::Any,
        guard: StepGuard::Always,
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
};
const CLOUD: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[END_TRIGGER],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::SourceAndAttachedEquipment,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
const EQUIPMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[END_TRIGGER],
    ..CardRules::EMPTY
};

fn packets(game: &mut Game, source: CardId) -> Vec<SpellView> {
    game.expand_trigger_multiplicity();
    let mut rows: Vec<_> = game
        .state
        .stack
        .iter()
        .chain(&game.state.pending_triggers)
        .chain(&game.state.ordering_triggers)
        .filter(|item| {
            item.source
                .is_some_and(|source_view| source_view.id == source)
        })
        .cloned()
        .collect();
    rows.sort_unstable_by_key(|item| item.id);
    assert!(rows.windows(2).all(|pair| pair[0].id != pair[1].id));
    assert!(rows.iter().all(|item| !item.is_copy));
    rows
}

fn end_step(game: &mut Game) {
    game.state.step = Step::Main2;
    game.give_priority();
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.step, Step::End);
}

#[test]
fn equipped_source_and_even_foreign_attached_equipment_each_double() {
    let cloud = Card::from_name("Cloud, Midgar Mercenary").unwrap();
    let equipment = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[(cloud, &CLOUD), (equipment, &EQUIPMENT)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, cloud, Zone::Battlefield);
    let foreign = spawn(&mut game, opponent, equipment, Zone::Battlefield);
    let loose = spawn(&mut game, player, equipment, Zone::Battlefield);
    let source_ref = game.object_ref(source);
    let mut unequipped = game.clone();
    assert!(game.attach_equipment(game.object_ref(foreign), source_ref));
    let mut blanked = game.clone();
    blanked.remove_abilities_exact(
        &[source_ref],
        source_ref,
        player,
        AnimationDuration::EndOfTurn,
    );

    end_step(&mut game);
    assert_eq!(packets(&mut game, source).len(), 2);
    let opposing = packets(&mut game, foreign);
    assert_eq!(opposing.len(), 2);
    assert!(opposing.iter().all(|item| item.controller == opponent));
    assert_eq!(packets(&mut game, loose).len(), 1);

    end_step(&mut unequipped);
    assert_eq!(packets(&mut unequipped, source).len(), 1);
    assert_eq!(packets(&mut unequipped, foreign).len(), 1);
    end_step(&mut blanked);
    assert!(packets(&mut blanked, source).is_empty());
    assert_eq!(packets(&mut blanked, foreign).len(), 1);
}

const BIRD: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        ..END_TRIGGER
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledLandOrSubtypeEntry("Bird"),
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
const WATCHER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PermanentEntersYou {
                kinds: 1 | 2,
                nontoken: true,
                subtype: None,
                include_source: false,
            },
            ..END_TRIGGER
        },
        TriggeredAbility {
            condition: TriggerCondition::TokensEnterYou,
            ..END_TRIGGER
        },
    ],
    ..CardRules::EMPTY
};

fn enter_together(game: &mut Game, entrants: &[CardId]) {
    let moves = entrants
        .iter()
        .map(|&entrant| game.entry_move(entrant, Zone::Battlefield, false))
        .collect();
    game.stage_entries(moves, None);
    while let Some(action) = game
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::ChooseEntryTimestamp { .. }))
    {
        act(game, action);
    }
}

#[test]
fn a_simultaneous_bird_or_land_entry_counts_each_provider_once() {
    let bird = Card::from_name("Traveling Chocobo").unwrap();
    let observer = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[(bird, &BIRD), (observer, &WATCHER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let monitor = spawn(&mut game, player, observer, Zone::Battlefield);
    let first = spawn(&mut game, player, bird, Zone::Hand);
    let second = spawn(&mut game, player, bird, Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let rival = spawn(&mut game, player.other(), bird, Zone::Hand);
    enter_together(&mut game, &[first, second, land, creature, rival]);

    assert_eq!(packets(&mut game, first).len(), 3);
    assert_eq!(packets(&mut game, second).len(), 3);
    assert_eq!(packets(&mut game, rival).len(), 2);
    let watched = packets(&mut game, monitor);
    for (entrant, count) in [
        (first, 3),
        (second, 3),
        (land, 3),
        (creature, 1),
        (rival, 0),
    ] {
        let reference = game.object_ref(entrant);
        assert_eq!(
            watched
                .iter()
                .filter(|item| item.entering_object == Some(reference))
                .count(),
            count
        );
    }
}

#[test]
fn two_birds_in_one_token_batch_do_not_multiply_the_batch_twice() {
    let bird = Card::from_name("Traveling Chocobo").unwrap();
    let observer = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[(bird, &BIRD), (observer, &WATCHER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let monitor = spawn(&mut game, player, observer, Zone::Battlefield);
    spawn(&mut game, player, bird, Zone::Battlefield);
    let start = game.state.objects.len() as CardId;
    game.create_tokens(player, Card::from_catalogue_id(5269), 2, false);
    for id in [start, start + 1] {
        assert!(game.object_view(id).has_subtype("Bird"));
    }
    assert_eq!(packets(&mut game, monitor).len(), 2);
}
