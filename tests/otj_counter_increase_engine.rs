//! Resolved counter replacements share the ordinary entry and placement order.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::replacements::{CounterPlacement, ResumeMode};
use rustmage_core::engine::*;
use support::*;

const INCREASER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::IncreaseOwnCreatureCountersThisTurn {
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const DOUBLE: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Creature,
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

fn resolve_increase(game: &mut Game, card: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    settle(game);
}

fn place(game: &mut Game, card: CardId, by: Player, kind: CounterKind, amount: u64) {
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
            amount: Counters::of(kind, amount),
            effect: true,
            entry_index: None,
            private: false,
        }],
        ResumeMode::Pending,
    );
    game.advance_replacements(true);
}

#[test]
fn two_resolutions_stack_only_for_their_controller_and_expire_at_cleanup() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &INCREASER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let creature = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let other = put(&mut game, opponent, "Savannah Lions", Zone::Battlefield);
    resolve_increase(&mut game, source);
    resolve_increase(&mut game, source);
    game.move_object(source, Zone::Graveyard);

    place(&mut game, artifact, player, CounterKind::PlusOne, 1);
    place(&mut game, other, player, CounterKind::PlusOne, 1);
    place(&mut game, creature, opponent, CounterKind::PlusOne, 1);
    place(&mut game, creature, player, CounterKind::Lifelink, 1);
    assert_eq!(game.counter_count(artifact, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(other, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(creature, CounterKind::Lifelink), 1);

    place(&mut game, creature, player, CounterKind::PlusOne, 1);
    let options = game
        .observe(player)
        .replacement_choice
        .unwrap()
        .resolved_counter_effects;
    assert_eq!(options.len(), 2);
    assert_ne!(options[0].id, options[1].id);
    act(
        &mut game,
        Action::ChooseResolvedCounterReplacement {
            effect: options[1].id,
        },
    );
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 4);

    game.state.step = Step::End;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(game.observe(player).creature_counter_increases.is_empty());
    place(&mut game, creature, player, CounterKind::PlusOne, 1);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 5);
}

#[test]
fn printed_and_resolved_effects_share_ordering_on_real_and_projected_entry() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &INCREASER),
        (Card::from_name("Short Bow").unwrap(), &DOUBLER),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let doubler = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let entrant = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
    resolve_increase(&mut game, source);
    let effect = game.observe(player).creature_counter_increases[0].id;
    let mut movement = game.entry_move(entrant, Zone::Battlefield, false);
    movement.controller = player;
    movement.effect_counter = Some((CounterKind::PlusOne, 2));

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
        projected.push(next.counter_count(entrant, CounterKind::PlusOne));
        false
    });
    projected.sort_unstable();
    projected.dedup();
    assert_eq!(projected, [5, 6]);

    game.stage_entries(vec![movement], None);
    let printed = Action::ChooseReplacement {
        source: game.object_ref(doubler),
        ability: 0,
    };
    let resolved = Action::ChooseResolvedCounterReplacement { effect };
    let mut indexed = game.clone();
    let checkpoint = indexed.snapshot();
    let menu = indexed.legal_actions();
    let index = menu.iter().position(|action| *action == resolved).unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.counter_count(entrant, CounterKind::PlusOne), 6);
    indexed.restore(&checkpoint).unwrap();
    act(&mut game, printed.clone());
    act(&mut indexed, printed);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.counter_count(entrant, CounterKind::PlusOne), 5);
}
