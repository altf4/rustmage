//! A stationed source grants flying only to its controller's live artifact creatures.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};

const STATIONED: StatValue = StatValue::AtLeast {
    value: &StatValue::SourceCounters(CounterKind::Charge),
    minimum: 12,
};
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(6),
    toughness: Some(6),
    own_turn: false,
    condition: Some(STATIONED),
};
const CUBE_RULE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Type,
            fragments: &[
                StaticAbilityFragment::SourceCounterCreature {
                    own_turn: false,
                    kind: CounterKind::Charge,
                    minimum: 12,
                    form: SourceCreatureForm::AddCreature,
                    subtypes: &[],
                    keywords: Keywords::NONE,
                },
                StaticAbilityFragment::BaseStatSetter(0, BASE),
            ],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::ControlledPermanentAllKindsKeywordsAtSourceCounters {
                    kinds: 1 | 16,
                    kind: CounterKind::Charge,
                    minimum: 12,
                    keywords: Keywords::FLYING,
                },
            ],
        },
    ],
    // Retain the source in speculative funding worlds for a real mana action.
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::any_color(1))],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ADD_CHARGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::Charge,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
const REMOVE_CHARGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::RemoveCounter {
            subject: Subject::Targets,
            kind: CounterKind::Charge,
        }],
    ),
    ..CardRules::EMPTY
};

fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 1,
        reserve: None,
        source: None,
    }
}

fn flying(game: &Game, id: CardId) -> bool {
    game.keywords(id).contains(Keywords::FLYING)
}

#[test]
fn station_flying_tracks_charge_count_live_types_and_exact_source() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Class-F Shuttlecraft").unwrap(), &CUBE_RULE),
        (Card::from_name("Opt").unwrap(), &ADD_CHARGE),
        (Card::from_name("Unsummon").unwrap(), &REMOVE_CHARGE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Class-F Shuttlecraft", Zone::Battlefield);
        let recipient = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Gingerbrute", Zone::Battlefield);
        game.set_counters(source, Counters::of(CounterKind::Charge, 11));
        assert!(!game.is_creature(source));
        for id in [source, recipient, creature, artifact, enemy] {
            assert!(!flying(&game, id));
        }

        let spell = put(&mut game, player, "Opt", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        let target = Action::ChooseTarget {
            target: Target::Card(source),
        };
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let action_index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == target)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), action_index)
            .unwrap();
        act(&mut game, target.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, target);
        assert_eq!(game.snapshot(), indexed.snapshot());
        pay(&mut game);
        pay(&mut indexed);
        settle(&mut game);
        settle(&mut indexed);
        assert_eq!(game.snapshot(), indexed.snapshot());
        drop(indexed);
        assert_eq!(game.counter_count(source, CounterKind::Charge), 12);
        assert!(game.is_creature(source));
        assert!(flying(&game, source));
        assert!(flying(&game, recipient));
        for id in [creature, artifact, enemy] {
            assert!(!flying(&game, id));
        }

        let mut changed_type = game.clone();
        changed_type.change_object_form(
            changed_type.object_ref(recipient),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&[]),
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            AnimationTypeMode::ReplaceWithCreature,
        );
        assert!(!flying(&changed_type, recipient));
        changed_type.expire_animations();
        assert!(flying(&changed_type, recipient));

        let mut changed_control = game.clone();
        changed_control.change_controller(changed_control.object_ref(recipient), player.other());
        assert!(!flying(&changed_control, recipient));
        changed_control.change_controller(changed_control.object_ref(source), player.other());
        assert!(flying(&changed_control, recipient));
        assert!(flying(&changed_control, source));

        let mut silenced = game.clone();
        let source_ref = silenced.object_ref(source);
        silenced.remove_abilities_exact(
            &[source_ref],
            source_ref,
            player,
            AnimationDuration::Permanent,
        );
        assert!(!flying(&silenced, recipient));

        let mut departed = game.clone();
        departed.move_object(source, Zone::Graveyard);
        assert!(!flying(&departed, recipient));
        departed.move_object(source, Zone::Battlefield);
        assert!(!flying(&departed, recipient));
        assert_ne!(departed.object_ref(source), game.object_ref(source));

        let spell = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(source)));
        settle(&mut game);
        assert_eq!(game.counter_count(source, CounterKind::Charge), 11);
        assert!(!game.is_creature(source));
        assert!(!flying(&game, recipient));
    }
}

#[test]
fn projected_station_flying_uses_owned_counter_and_controller_values() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Class-F Shuttlecraft").unwrap(), &CUBE_RULE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Class-F Shuttlecraft", Zone::Battlefield);
    let recipient = put(&mut game, player, "Forest", Zone::Battlefield);
    game.change_object_form(
        game.object_ref(recipient),
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        AnimationTypeMode::AddCreature,
    );
    game.set_counters(source, Counters::of(CounterKind::Charge, 11));
    assert!(!flying(&game, recipient));
    let mut twelve = game.clone();
    twelve.set_counters(source, Counters::of(CounterKind::Charge, 12));
    let mut world = FundingState::from_game(&game, &goal(player));
    world.named_counters = twelve.state.named_counters.clone();
    assert!(
        world
            .activation_keywords(recipient)
            .contains(Keywords::FLYING)
    );
    world
        .resources
        .iter_mut()
        .find(|row| row.id == recipient)
        .unwrap()
        .object
        .controller = player.other();
    assert!(
        !world
            .activation_keywords(recipient)
            .contains(Keywords::FLYING)
    );
    assert!(
        !flying(&game, recipient),
        "projection must not mutate parent"
    );

    game.set_counters(source, Counters::of(CounterKind::Charge, 12));
    assert!(flying(&game, recipient));
    let mut world = FundingState::from_game(&game, &goal(player));
    world.named_counters = {
        let mut eleven = game.clone();
        eleven.set_counters(source, Counters::of(CounterKind::Charge, 11));
        eleven.state.named_counters.clone()
    };
    assert!(
        !world
            .activation_keywords(recipient)
            .contains(Keywords::FLYING)
    );
    assert!(flying(&game, recipient));
}
