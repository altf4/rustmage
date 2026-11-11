//! Chosen-type multipliers count once at the triggering event.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::ChosenCreatureType],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::OtherControlledCreatureOfChosenType,
                condition: StatValue::Fixed(1),
            }],
        },
    ],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ENTRY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const CAST_EVENT: TriggerCondition = TriggerCondition::SpellCast {
    relation: CastRelation::You,
    filter: CastFilter::InstantSorcery,
    ordinal: None,
};
const TARGETED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: CAST_EVENT,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::Damage(1)],
    }],
    ..CardRules::EMPTY
};
const ATTACKS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::SelfAttacks),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 10)],
        },
    ],
    ..CardRules::EMPTY
};
const DEPARTURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCreatureDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const CAST_OBSERVER: TriggeredAbility = TriggeredAbility {
    condition: CAST_EVENT,
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[CAST_OBSERVER],
    ..CardRules::EMPTY
};
const KINDRED_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::CHANGELING,
    triggers: &[CAST_OBSERVER],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::Changeling],
    }],
    ..CardRules::EMPTY
};
const CHANGE_TYPE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::SetCreatureTypes {
            subject: Subject::Targets,
            types: &["Goblin"],
        }],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn choose_type(name: &str) -> Action {
    Action::ChooseEntryValue {
        value: EntryChoiceValue::CreatureType(CreatureType::from_name(name).unwrap()),
    }
}

fn cast_to_choice(game: &mut Game, player: Player, name: &str) -> CardId {
    let spell = put(game, player, name, Zone::Hand);
    cast(game, spell, None);
    pass_priority(game);
    assert!(matches!(game.state.pending, Pending::EntryValue { .. }));
    spell
}

fn settle_untargeted(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    settle(game);
}

fn provider(game: &mut Game, player: Player, kind: &str) -> CardId {
    let id = cast_to_choice(game, player, "Gingerbrute");
    act(game, choose_type(kind));
    settle_untargeted(game);
    id
}

fn packets(game: &Game, source: ObjectRef, ability: u16) -> Vec<u64> {
    game.state
        .pending_triggers
        .iter()
        .chain(&game.state.ordering_triggers)
        .chain(&game.state.stack)
        .filter(|item| {
            item.kind == StackKind::TriggeredAbility { ability }
                && item.source.is_some_and(|s| s.object == source)
                && !item.is_copy
        })
        .map(|item| item.id)
        .collect()
}

fn shock(game: &mut Game, player: Player) {
    let spell = put(game, player, "Shock", Zone::Hand);
    cast(game, spell, Some(Target::Player(player.other())));
}

#[test]
fn owned_type_choice_combines_once_with_the_entry_scope() {
    let _rules = fixture_rules::activate(&[
        (named("Gingerbrute"), &PROVIDER),
        (named("Bear Cub"), &ENTRY),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        let provider = cast_to_choice(&mut root, player, "Gingerbrute");
        let action = choose_type("Bear");
        let revision = root.revision();
        let index = root
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let checkpoint = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);

        act(&mut raw, action);
        indexed.apply_index(player, revision, index).unwrap();
        settle_untargeted(&mut raw);
        settle_untargeted(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[player.index()].life, STARTING_LIFE + 1);
        assert!(raw.has_subtype(provider, "Golem"));
        assert!(raw.has_subtype(provider, "Bear"));

        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, choose_type("Goblin"));
        settle_untargeted(&mut indexed);
        assert!(indexed.has_subtype(provider, "Goblin"));
        assert!(!indexed.has_subtype(provider, "Bear"));
        assert!(indexed.has_subtype(provider, "Golem"));

        for (mut game, expected) in [(raw, 3), (indexed, 2)] {
            put(&mut game, player, "Starfield Vocalist", Zone::Battlefield);
            let bear = put(&mut game, player, "Bear Cub", Zone::Hand);
            cast(&mut game, bear, None);
            pass_priority(&mut game);
            assert_eq!(packets(&game, game.object_ref(bear), 0).len(), expected);
            settle_untargeted(&mut game);
            assert_eq!(
                game.state.players[player.index()].life,
                STARTING_LIFE + 1 + expected as i128
            );
        }
    }
}

#[test]
fn multiple_providers_exclude_themselves_and_ordinary_triggers_choose_independently() {
    let _rules = fixture_rules::activate(&[
        (named("Gingerbrute"), &PROVIDER),
        (named("Bear Cub"), &TARGETED),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        provider(&mut game, player, "Bear");
        provider(&mut game, player, "Bear");
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 3);
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let first = put(&mut game, player, "Gigantosaurus", Zone::Battlefield);
        let second = put(&mut game, opponent, "Magnigoth Sentry", Zone::Battlefield);
        shock(&mut game, player);

        assert_eq!(game.actor(), Some(player));
        let ids = packets(&game, game.object_ref(bear), 0);
        assert_eq!(ids.len(), 3);
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[0], ids[2]);
        assert_ne!(ids[1], ids[2]);
        act(&mut game, Action::PutTrigger { id: ids[2] });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        act(&mut game, Action::PutTrigger { id: ids[0] });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(second),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        settle(&mut game);
        assert_eq!(game.object_view(first).damage, 2);
        assert_eq!(game.object_view(second).damage, 1);
    }
}

#[test]
fn attack_scope_composes_and_printed_once_per_turn_remains_one() {
    let _rules = fixture_rules::activate(&[
        (named("Gingerbrute"), &PROVIDER),
        (named("Bear Cub"), &ATTACKS),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        provider(&mut game, player, "Bear");
        cast_to_choice(&mut game, player, "Windcrag Siege");
        act(
            &mut game,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::NamedMode(0),
            },
        );
        settle(&mut game);
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        pass_priority(&mut game);
        pass_priority(&mut game);
        act(&mut game, Action::Attack { card: bear });
        act(&mut game, Action::FinishAttackers);

        let exact = game.object_ref(bear);
        assert_eq!(packets(&game, exact, 0).len(), 3);
        assert_eq!(packets(&game, exact, 1).len(), 1);
        settle_untargeted(&mut game);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 14);
    }
}

#[test]
fn simultaneous_departure_uses_the_provider_and_creature_before_the_event() {
    let _rules = fixture_rules::activate(&[
        (named("Gingerbrute"), &PROVIDER),
        (named("Bear Cub"), &DEPARTURE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        let provider = provider(&mut root, player, "Bear");
        let bear = put(&mut root, player, "Bear Cub", Zone::Battlefield);
        let exact = root.object_ref(bear);
        let sweep = put(&mut root, player, "Day of Judgment", Zone::Hand);
        let mut early = root.clone();
        early.move_object(provider, Zone::Graveyard);

        for (mut game, expected) in [(root, 2), (early, 1)] {
            cast(&mut game, sweep, None);
            pass_priority(&mut game);
            assert_eq!(game.object_view(provider).zone, Zone::Graveyard);
            assert_eq!(game.object_view(bear).zone, Zone::Graveyard);
            assert_eq!(packets(&game, exact, 0).len(), expected);
            settle_untargeted(&mut game);
            assert_eq!(
                game.state.players[player.index()].life,
                STARTING_LIFE + 1 + expected as i128
            );
        }
    }
}

#[test]
fn the_live_type_controller_and_ability_are_checked_for_each_event() {
    let _rules = fixture_rules::activate(&[
        (named("Gingerbrute"), &PROVIDER),
        (named("Bear Cub"), &OBSERVER),
        (named("Firdoch Core"), &KINDRED_OBSERVER),
        (named("Giant Growth"), &CHANGE_TYPE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let provider = provider(&mut game, player, "Bear");
        let provider_ref = game.object_ref(provider);
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let bear_ref = game.object_ref(bear);
        let kindred = put(&mut game, player, "Firdoch Core", Zone::Battlefield);
        let kindred_ref = game.object_ref(kindred);
        assert!(game.has_subtype(kindred, "Bear"));
        assert!(!game.is_creature(kindred));

        let check = |game: &mut Game, expected| {
            shock(game, player);
            assert_eq!(packets(game, bear_ref, 0).len(), expected);
            assert_eq!(packets(game, kindred_ref, 0).len(), 1);
            settle_untargeted(game);
        };
        check(&mut game, 2);
        let type_change = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, type_change, Some(Target::Card(bear)));
        settle_untargeted(&mut game);
        assert!(game.has_subtype(bear, "Goblin"));
        assert!(!game.has_subtype(bear, "Bear"));
        check(&mut game, 1);

        game.forget_creature_types(bear_ref);
        game.record_control(provider_ref, player.other(), None);
        check(&mut game, 1);
        game.record_control(provider_ref, player, None);

        let aura = put(&mut game, player, "Witness Protection", Zone::Hand);
        cast(&mut game, aura, Some(Target::Card(provider)));
        settle(&mut game);
        check(&mut game, 1);
        game.move_object(aura, Zone::Graveyard);
        check(&mut game, 2);
    }
}

#[test]
fn acquiring_the_same_provider_ability_does_not_inherit_an_entry_choice() {
    let _rules = fixture_rules::activate(&[
        (named("Gingerbrute"), &PROVIDER),
        (named("Bear Cub"), &OBSERVER),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let original = provider(&mut game, player, "Bear");
        let original_ref = game.object_ref(original);
        let observer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let observer_ref = game.object_ref(observer);
        let mimic = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let mimic_ref = game.object_ref(mimic);
        let recipe = game.copiable_recipe(original_ref);
        game.apply_temporary_copy(mimic_ref, recipe, CopyDeadline::EndOfTurn);
        assert!(game.has_subtype(mimic, "Golem"));
        assert!(!game.has_subtype(mimic, "Bear"));
        assert!(game.entry_value(mimic_ref).is_none());
        shock(&mut game, player);
        assert_eq!(packets(&game, observer_ref, 0).len(), 2);
        settle_untargeted(&mut game);

        game.apply_temporary_copy(
            original_ref,
            recipe,
            CopyDeadline::WhileSource { source: mimic_ref },
        );
        assert!(!game.has_subtype(original, "Bear"));
        assert!(game.entry_value(original_ref).is_none());
        shock(&mut game, player);
        assert_eq!(packets(&game, observer_ref, 0).len(), 1);
        settle_untargeted(&mut game);

        game.move_object(mimic, Zone::Graveyard);
        assert!(game.has_subtype(original, "Bear"));
        shock(&mut game, player);
        assert_eq!(packets(&game, observer_ref, 0).len(), 2);
        settle_untargeted(&mut game);
    }
}
