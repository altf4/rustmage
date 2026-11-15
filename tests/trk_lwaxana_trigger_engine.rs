//! Exact tapped-source duration for extra triggered ability instances.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OWN_OTHER_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: true,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const LWAXANA: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::MaySkipOwnUntap],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(OWN_OTHER_CREATURE)),
        program: &[Instruction::DoubleTargetTriggersWhileSourceTapped],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const BEARER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const UNTAP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Untap],
    ),
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::GrizzlyBears, &LWAXANA),
        (Card::HillGiant, &BEARER),
        (Card::from_name("Opt").unwrap(), &UNTAP),
    ])
}

fn activate_source(game: &mut Game, source: CardId, target: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(game);
}

fn activate_bearer(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(game);
    drain(game);
}

fn drain(game: &mut Game) {
    for _ in 0..48 {
        match game.state.pending {
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
            Pending::Priority
                if game.state.stack.is_empty()
                    && game.state.resolving.is_none()
                    && game.state.pending_triggers.is_empty() =>
            {
                return;
            }
            Pending::Priority => act(game, Action::Pass),
            ref other => panic!("unexpected trigger state {other:?}"),
        }
    }
    panic!("trigger resolution exceeded 48 actions");
}

#[test]
fn paid_target_choice_doubles_only_that_creatures_real_triggers_and_branches_restore() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let source = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
        let bearer = put(&mut parent, player, "Hill Giant", Zone::Battlefield);
        let other = put(&mut parent, player, "Hill Giant", Zone::Battlefield);
        let opposing = put(&mut parent, player.other(), "Hill Giant", Zone::Battlefield);
        act(
            &mut parent,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut parent,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        reject(
            &mut parent,
            Action::ChooseTarget {
                target: Target::Card(opposing),
            },
        );
        act(
            &mut parent,
            Action::ChooseTarget {
                target: Target::Card(bearer),
            },
        );
        pay(&mut parent);
        settle(&mut parent);
        assert!(parent.object_view(source).tapped);
        assert_eq!(
            parent.observe(player).source_linked_effects,
            parent.observe(player.other()).source_linked_effects
        );

        let action = Action::AnnounceActivate {
            card: bearer,
            ability: 0,
            counter_payment: Counters::default(),
        };
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let checkpoint = parent.snapshot();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        act(&mut raw, action);
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        pay(&mut raw);
        pay(&mut indexed);
        drain(&mut raw);
        drain(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[player.index()].life, 23);
        drop(parent);

        indexed.restore(&checkpoint).unwrap();
        activate_bearer(&mut indexed, bearer);
        assert_eq!(indexed.snapshot(), raw.snapshot());

        // A different creature's own tap trigger still occurs once.
        activate_bearer(&mut raw, other);
        assert_eq!(raw.state.players[player.index()].life, 25);
    }
}

#[test]
fn source_untap_before_or_after_resolution_ends_the_duration_forever() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let bearer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let untap = put(&mut game, player, "Opt", Zone::Hand);
    activate_source(&mut game, source, bearer);
    cast(&mut game, untap, Some(Target::Card(source)));
    settle(&mut game);
    assert!(!game.object_view(source).tapped);
    activate_bearer(&mut game, bearer);
    assert_eq!(game.state.players[player.index()].life, 22);

    let mut established = scenario(player);
    let source = put(&mut established, player, "Grizzly Bears", Zone::Battlefield);
    let bearer = put(&mut established, player, "Hill Giant", Zone::Battlefield);
    activate_source(&mut established, source, bearer);
    settle(&mut established);
    let untap = put(&mut established, player, "Opt", Zone::Hand);
    cast(&mut established, untap, Some(Target::Card(source)));
    settle(&mut established);
    assert!(!established.object_view(source).tapped);
    established.tap_objects_as(player, &[source]);
    activate_bearer(&mut established, bearer);
    assert_eq!(established.state.players[player.index()].life, 22);
}

#[test]
fn source_departure_or_target_death_uses_exact_lifetime_and_trigger_lki() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let bearer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    activate_source(&mut game, source, bearer);
    settle(&mut game);
    game.move_object(bearer, Zone::Graveyard);
    game.prepare_triggers();
    drain(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);

    let mut departed = scenario(player);
    let source = put(&mut departed, player, "Grizzly Bears", Zone::Battlefield);
    let bearer = put(&mut departed, player, "Hill Giant", Zone::Battlefield);
    activate_source(&mut departed, source, bearer);
    settle(&mut departed);
    departed.move_object(source, Zone::Graveyard);
    activate_bearer(&mut departed, bearer);
    assert_eq!(departed.state.players[player.index()].life, 22);
}

#[test]
fn optional_untap_choice_keeps_or_ends_the_effect() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let bearer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    activate_source(&mut game, source, bearer);
    settle(&mut game);
    game.begin_turn();
    assert_eq!(
        game.observe(player).untap_choice,
        Some(game.object_ref(source))
    );
    reject(&mut game, Action::Pass);
    let checkpoint = game.snapshot();
    act(&mut game, Action::ChooseOptional { accept: false });
    assert!(game.object_view(source).tapped);
    activate_bearer(&mut game, bearer);
    assert_eq!(game.state.players[player.index()].life, 23);

    game.restore(&checkpoint).unwrap();
    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(!game.object_view(source).tapped);
    activate_bearer(&mut game, bearer);
    assert_eq!(game.state.players[player.index()].life, 22);
}
