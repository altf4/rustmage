//! A resolved tap trigger increases every positive counter kind on controlled permanents.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::replacements::{CounterPlacement, ResumeMode};
use rustmage_core::{cards::*, engine::*};
use support::*;

const TRIGGER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesTapped,
        targets: TargetPlan::NONE,
        program: &[Instruction::IncreaseControlledPermanentCountersThisTurn { amount: 1 }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
const DOUBLE: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Permanent,
    kind: Some(CounterKind::PlusOne),
    factor: 2,
    add: 0,
    effects_only: false,
};
const DOUBLER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn tap_and_resolve(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    settle(game);
}

fn place(game: &mut Game, object: CardId, placed_by: Player, amount: Counters, effect: bool) {
    let view = game.object_view(object);
    game.stage_counter_placements(
        vec![CounterPlacement {
            object: view.reference,
            placed_by,
            card: view.card,
            owner: view.owner,
            controller: view.controller,
            protector: view.battle_protector,
            subtypes: view.granted_subtypes,
            kind: view.kinds,
            power: view.power,
            amount,
            effect,
            entry_index: None,
            private: false,
        }],
        ResumeMode::Pending,
    );
    game.advance_replacements(true);
    for _ in 0..8 {
        let Some(choice) = game.observe(game.state.active).replacement_choice else {
            break;
        };
        let Some(effect) = choice.resolved_permanent_counter_effects.first() else {
            break;
        };
        act(
            game,
            Action::ChooseResolvedCounterReplacement { effect: effect.id },
        );
    }
    assert!(game.observe(game.state.active).replacement_choice.is_none());
}

#[test]
fn tapped_trigger_stacks_for_every_kind_permanent_and_placer_then_expires() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TRIGGER)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        tap_and_resolve(&mut game, source);
        game.state.objects[source as usize].tapped = false;
        tap_and_resolve(&mut game, source);
        for viewer in [player, player.other()] {
            assert_eq!(game.observe(viewer).permanent_counter_increases.len(), 2);
        }
        game.move_object(source, Zone::Graveyard);

        let mut mixed = Counters::of(CounterKind::PlusOne, 2);
        mixed.add(CounterKind::Stun, 1);
        mixed.add(CounterKind::Charge, 3);
        place(&mut game, creature, player.other(), mixed, true);
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 4);
        assert_eq!(game.counter_count(creature, CounterKind::Stun), 3);
        assert_eq!(game.counter_count(creature, CounterKind::Charge), 5);
        place(
            &mut game,
            artifact,
            player.other(),
            Counters::of(CounterKind::Lore, 1),
            false,
        );
        assert_eq!(game.counter_count(artifact, CounterKind::Lore), 3);
        place(
            &mut game,
            land,
            player,
            Counters::of(CounterKind::Charge, 1),
            true,
        );
        assert_eq!(game.counter_count(land, CounterKind::Charge), 3);
        place(
            &mut game,
            opposing,
            player,
            Counters::of(CounterKind::PlusOne, 1),
            true,
        );
        assert_eq!(game.counter_count(opposing, CounterKind::PlusOne), 1);

        game.state.step = Step::End;
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert!(game.observe(player).permanent_counter_increases.is_empty());
        place(
            &mut game,
            artifact,
            player,
            Counters::of(CounterKind::Charge, 1),
            true,
        );
        assert_eq!(game.counter_count(artifact, CounterKind::Charge), 1);
    }
}

#[test]
fn entry_replacement_orders_with_doubler_in_real_and_projected_worlds() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &TRIGGER),
        (Card::from_name("Short Bow").unwrap(), &DOUBLER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let doubler = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let entrant = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
    tap_and_resolve(&mut game, source);
    let effect = game.observe(player).permanent_counter_increases[0].id;
    game.move_object(source, Zone::Graveyard);
    let mut movement = game.entry_move(entrant, Zone::Battlefield, false);
    movement.controller = player;
    movement.effect_counters = &[(CounterKind::PlusOne, 2), (CounterKind::Stun, 1)];

    let goal = FundingGoal {
        actor: player,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let world = FundingState::from_game(&game, &goal);
    let mut projected = Vec::new();
    world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
        projected.push((
            next.counter_count(entrant, CounterKind::PlusOne),
            next.counter_count(entrant, CounterKind::Stun),
        ));
        false
    });
    projected.sort_unstable();
    projected.dedup();
    assert_eq!(projected, [(5, 2), (6, 2)]);

    game.stage_entries(vec![movement], None);
    assert!(game.advance_entries());
    let printed = Action::ChooseReplacement {
        source: game.object_ref(doubler),
        ability: 0,
    };
    let resolved = Action::ChooseResolvedCounterReplacement { effect };
    assert_eq!(
        game.observe(player.other())
            .replacement_choice
            .as_ref()
            .unwrap()
            .resolved_permanent_counter_effects[0]
            .id,
        effect,
    );
    reject(
        &mut game,
        Action::ChooseResolvedCounterReplacement {
            effect: effect + 1000,
        },
    );
    let checkpoint = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == resolved)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.counter_count(entrant, CounterKind::PlusOne), 6);
    indexed.restore(&checkpoint).unwrap();
    act(&mut game, printed.clone());
    act(&mut indexed, printed);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.counter_count(entrant, CounterKind::PlusOne), 5);
    assert_eq!(game.counter_count(entrant, CounterKind::Stun), 2);
}
