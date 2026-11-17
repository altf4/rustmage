//! Exact, bounded source-power mana choices and their guided funding continuations.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ZERO: Mana = [0; 6];
const SPLIT_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::OwnTurn,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::SourcePowerSplit {
        first: ChoiceColor::Blue,
        second: ChoiceColor::Red,
    })],
    sorcery: false,
    once_per_turn: true,
};
const SPLITTER: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    activated: &[SPLIT_ABILITY],
    ..CardRules::EMPTY
};

fn start_mana(card: CardId) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: ZERO,
        counter_payment: Counters::default(),
        production: ZERO,
    }
}

fn choose_mana(blue: u32, red: u32) -> Action {
    Action::ChooseMana {
        production: [0, blue, 0, red, 0, 0],
    }
}

fn setup(player: Player) -> (Game, CardId) {
    let mut game = scenario(player);
    game.state.players[player.index()].mana = ZERO;
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[source as usize].summoning_sick = true;
    (game, source)
}

#[test]
fn zero_and_negative_power_are_valid_and_live_power_has_one_activation_per_own_turn() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SPLITTER)]);
    let player = Player::P1;
    let (mut parent, source) = setup(player);
    let start = start_mana(source);
    let mut off_turn = parent.clone();
    off_turn.state.active = player.other();
    reject(&mut off_turn, start.clone());

    for power in [0, -3] {
        let mut branch = parent.clone();
        branch.state.objects[source as usize].growth.power = power;
        act(&mut branch, start.clone());
        assert_eq!(branch.state.pending, Pending::Priority);
        assert!(branch.state.stack.is_empty());
        assert_eq!(branch.state.players[player.index()].mana, ZERO);
        assert!(!branch.object_view(source).tapped);
        reject(&mut branch, start.clone());
    }

    parent.set_counters(source, Counters::of(CounterKind::PlusOne, 3));
    act(&mut parent, start.clone());
    assert!(parent.state.stack.is_empty());
    assert_eq!(
        parent.observe(player).mana_choices,
        [
            [0, 0, 0, 3, 0, 0],
            [0, 1, 0, 2, 0, 0],
            [0, 2, 0, 1, 0, 0],
            [0, 3, 0, 0, 0, 0]
        ]
    );
    act(&mut parent, choose_mana(1, 2));
    assert_eq!(
        parent.state.players[player.index()].mana,
        [0, 1, 0, 2, 0, 0]
    );
    assert!(!parent.object_view(source).tapped);
    reject(&mut parent, start.clone());

    for _ in 0..2 {
        parent.state.step = Step::End;
        parent.give_priority();
        act(&mut parent, Action::Pass);
        act(&mut parent, Action::Pass);
    }
    assert_eq!(parent.state.active, player);
    act(&mut parent, start);
    act(&mut parent, choose_mana(0, 3));
    assert_eq!(
        parent.state.players[player.index()].mana,
        [0, 0, 0, 3, 0, 0]
    );
}

#[test]
fn wide_split_respects_component_capacity_with_exact_raw_and_bounded_index_actions() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SPLITTER)]);
    let player = Player::P0;
    let (mut parent, source) = setup(player);
    let total = u64::from(u32::MAX) + 1_000;
    parent.state.objects[source as usize].growth.power = i128::from(total);
    parent.state.players[player.index()].mana = [0, 13, 0, 23, 0, 0];
    act(&mut parent, start_mana(source));
    let space = parent.legal_action_space();
    let domain = space.mana_split.unwrap();
    assert_eq!(domain.first, ChoiceColor::Blue);
    assert_eq!(domain.second, ChoiceColor::Red);
    assert_eq!(domain.total, total);
    assert_eq!(domain.minimum_first, 1_023);
    assert_eq!(domain.maximum_first, u32::MAX - 13);
    assert!(!domain.guided);
    assert_eq!(parent.observe(player).mana_split, Some(domain));
    assert!(parent.observe(player.other()).mana_split.is_none());
    assert!(parent.observe(player.other()).mana_choices.is_empty());
    assert!(space.choices.len() <= 4);

    let over_capacity = choose_mana(1_000, u32::MAX);
    reject(&mut parent, over_capacity);
    reject(&mut parent, choose_mana(100, 200));
    reject(
        &mut parent,
        Action::ChooseMana {
            production: [0, 1_023, 1, u32::MAX - 23, 0, 0],
        },
    );

    let amount = 123_456;
    let remainder = u32::try_from(total - u64::from(amount)).unwrap();
    let arbitrary = choose_mana(amount, remainder);
    assert!(parent.mana_split_production_legal([0, amount, 0, remainder, 0, 0]));
    assert!(!space.choices.contains(&arbitrary));
    let (index, sample) = space
        .choices
        .into_iter()
        .enumerate()
        .find(|(_, action)| matches!(action, Action::ChooseMana { .. }))
        .unwrap();
    let revision = parent.revision();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, sample);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.state.players[player.index()].mana[3], u32::MAX);

    raw.restore(&saved).unwrap();
    act(&mut raw, arbitrary);
    assert_eq!(
        raw.state.players[player.index()].mana,
        [0, amount + 13, 0, remainder + 23, 0, 0]
    );
    assert_eq!(raw.state.pending, Pending::Priority);

    let (mut overflow, source) = setup(player);
    overflow.state.objects[source as usize].growth.power = i128::from(u32::MAX) * 2 + 1;
    let saved = overflow.snapshot();
    let revision = overflow.revision();
    assert!(matches!(
        overflow.apply(player, start_mana(source)),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(overflow.snapshot(), saved);
    assert_eq!(overflow.revision(), revision);
}

const BLUE_FILTER: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        once_per_turn: false,
        cost: ActivationCost {
            colored: [0, 2, 0, 0, 0, 0],
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        ..SPLIT_ABILITY
    }],
    ..CardRules::EMPTY
};
const RED_FILTER: CardRules = CardRules {
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        once_per_turn: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 2, 0, 0],
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 1, 0,
        ]))],
        ..SPLIT_ABILITY
    }],
    ..BLUE_FILTER
};

#[test]
fn split_choices_do_not_filter_parent_payment_continuations() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SPLITTER),
        (Card::from_name("Gilded Lotus").unwrap(), &BLUE_FILTER),
        (Card::from_name("Swiftfoot Boots").unwrap(), &RED_FILTER),
    ]);
    let player = Player::P0;
    let (mut game, source) = setup(player);
    game.state.objects[source as usize].growth.power = 2;
    let blue = put(&mut game, player, "Gilded Lotus", Zone::Battlefield);
    let red = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let spell = put(&mut game, player, "Llanowar Elves", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, start_mana(source));
    let space = game.legal_action_space();
    let domain = space.mana_split.unwrap();
    assert!(!domain.guided);
    let mixed = [0, 1, 0, 1, 0, 0];
    assert!(domain.contains_allocation(mixed));
    assert!(game.mana_split_production_legal(mixed));
    assert!(space.choices.contains(&choose_mana(0, 2)));
    assert!(space.choices.contains(&choose_mana(2, 0)));
    admit(&game, choose_mana(1, 1));
    let mut unfunded = game.clone();
    act(&mut unfunded, choose_mana(1, 1));
    act(&mut unfunded, Action::EndManaWindow);
    act(&mut unfunded, Action::SubmitAttempt);
    assert_eq!(
        unfunded.last_attempt_outcome_view(player).unwrap().reason,
        Some(AttemptFailureReason::UnsatisfiedMana)
    );

    act(&mut game, choose_mana(2, 0));
    assert_eq!(game.state.pending, Pending::PayCosts { player });
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, .. } if *card == blue),
    );
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(blue).tapped);
    assert!(!game.object_view(red).tapped);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
}

#[test]
fn large_split_validates_the_supplied_total_without_planning_future_mana() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SPLITTER)]);
    let player = Player::P0;
    let (mut game, source) = setup(player);
    game.state.objects[source as usize].growth.power = 1_000_001;
    Game::reset_payment_planning_work();
    act(&mut game, start_mana(source));
    let space = game.legal_action_space();
    let domain = space.mana_split.unwrap();
    assert_eq!(domain.total, 1_000_001);
    assert!(!domain.guided);
    assert!(space.choices.len() <= 4);
    reject(&mut game, choose_mana(500_001, 500_001));
    act(&mut game, choose_mana(500_000, 500_001));
    assert_eq!(
        game.state.players[player.index()].mana,
        [0, 500_000, 0, 500_001, 0, 0]
    );
    assert!(!game.object_view(source).tapped);
    let work = Game::payment_planning_work();
    assert_eq!(
        work.projected_moves + work.projected_steps + work.continuation_plans,
        0
    );
}
