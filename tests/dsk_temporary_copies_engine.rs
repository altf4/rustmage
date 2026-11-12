//! Exact exile receipts, simultaneous copies and expiration at the next end step.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const FOOD: Card = Card::from_catalogue_id(5246);
const COPY_ABILITY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
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
        targets: one(TargetRule::Filtered {
            base: &TargetRule::creature_you_control(),
            predicate: &TargetPredicate::Nonlegendary,
        }),
        // The preexisting Food subtype lets this exercise the engine independently.
        program: &[Instruction::ExileTargetAndCopyToSubtypeUntilEndStep { subtype: "Food" }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const END_TRIGGER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn token(game: &mut Game, player: Player, card: Card, tapped: bool) -> ObjectRef {
    let id = game.state.objects.len() as CardId;
    game.create_tokens(player, card, 1, tapped);
    game.object_ref(id)
}

fn view(game: &Game, id: CardId) -> ObjectView {
    game.observe(PLAYER)
        .objects
        .into_iter()
        .find(|object| object.id == id)
        .unwrap()
}

fn start_activation(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn choose_and_pay(game: &mut Game, chosen: CardId) {
    act(game, target(chosen));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn renamed(card: Card, name: &'static str) -> CopyRecipe {
    let mut recipe = CopyRecipe::new(card);
    recipe.overrides = Some(CopyOverrides {
        name: Some(name),
        ..CopyOverrides::NONE
    });
    recipe
}

#[test]
fn copies_the_frozen_cohort_and_returns_the_physical_card_to_its_owner() {
    let giant = named("Hill Giant");
    let _rules =
        fixture_rules::activate(&[(named("Bear Cub"), &COPY_ABILITY), (giant, &END_TRIGGER)]);
    let mut game = scenario(PLAYER);
    game.state.step = Step::Main2;
    let opponent = PLAYER.other();
    let source = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let target_creature = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let unrelated = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    let legendary = put(
        &mut game,
        PLAYER,
        "Ghalta, Primal Hunger",
        Zone::Battlefield,
    );
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    let first = token(&mut game, PLAYER, FOOD, true);
    let second = token(&mut game, PLAYER, FOOD, false);
    let opposing_food = token(&mut game, opponent, FOOD, false);
    let blueprint = game.object_ref(target_creature);
    game.record_control(blueprint, PLAYER, None);
    game.apply_temporary_copy(
        blueprint,
        renamed(giant, "Ancient Giant"),
        CopyDeadline::EndOfTurn,
    );
    game.add_counter(target_creature, CounterKind::PlusOne, 2);
    game.apply_temporary_copy(
        first,
        renamed(FOOD, "Pantry"),
        CopyDeadline::WhileSource { source: first },
    );
    let original_layer = view(&game, first.card)
        .temporary_copy
        .unwrap()
        .active_origin;
    assert_eq!(
        (
            view(&game, target_creature).power,
            view(&game, target_creature).toughness
        ),
        (5, 5)
    );

    start_activation(&mut game, source);
    reject(&mut game, target(unrelated));
    reject(&mut game, target(legendary));
    choose_and_pay(&mut game, target_creature);
    assert!(view(&game, source).tapped);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));
    cast(&mut game, bounce, Some(Target::Card(source)));
    settle(&mut game);

    let exiled = view(&game, target_creature);
    assert_eq!(exiled.zone, Zone::Exile);
    assert_ne!(exiled.reference, blueprint);
    assert_eq!(view(&game, source).zone, Zone::Hand);
    assert_eq!(view(&game, opposing_food.card).card, FOOD);
    let observation = game.observe(opponent);
    let [delayed] = &observation.delayed_returns[..] else {
        panic!("expected one delayed return");
    };
    assert_eq!(delayed.controller, PLAYER);
    assert_eq!(delayed.returning.controller, opponent);
    assert_eq!(delayed.returning.object, exiled.reference);
    for (recipient, tapped) in [(first, true), (second, false)] {
        let copy = view(&game, recipient.card);
        assert_eq!(copy.reference, recipient);
        assert_eq!(copy.name, "Ancient Giant");
        assert!(copy.is_token() && copy.is_creature() && copy.has_subtype("Giant"));
        assert!(!copy.has_subtype("Food"));
        assert_eq!((copy.power, copy.toughness), (3, 3));
        assert!(!copy.counters.has_any());
        assert_eq!(copy.tapped, tapped);
    }
    let first_copy = view(&game, first.card).temporary_copy.unwrap();
    let second_copy = view(&game, second.card).temporary_copy.unwrap();
    assert_eq!(first_copy.layers[0].origin, original_layer);
    assert_ne!(first_copy.active_origin, second_copy.active_origin);
    assert_eq!(
        first_copy.layers[1].provider_timestamp,
        second_copy.layers[0].provider_timestamp
    );
    assert_eq!(first_copy.layers[1].deadline, CopyDeadline::NextEndStep);

    let later = token(&mut game, PLAYER, FOOD, false);
    game.record_control(first, opponent, None);
    assert_eq!(view(&game, first.card).name, "Ancient Giant");
    assert_eq!(view(&game, later.card).card, FOOD);
    pass_pair(&mut game);
    let end = game.observe(PLAYER);
    assert_eq!(end.step, Step::End);
    let [returning] = &end.stack[..] else {
        panic!("copies must expire before end-step triggers are checked");
    };
    assert_eq!(returning.kind, StackKind::DelayedReturn);
    assert_eq!(view(&game, first.card).name, "Pantry");
    assert_eq!(view(&game, first.card).controller, opponent);
    assert_eq!(view(&game, second.card).card, FOOD);
    assert!(view(&game, second.card).temporary_copy.is_none());
    settle(&mut game);
    let returned = view(&game, target_creature);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.card, Card::GrizzlyBears);
    assert_eq!(returned.controller, opponent);
    assert_ne!(returned.reference, exiled.reference);
    assert!(!returned.counters.has_any());
}

#[test]
fn an_inserted_end_step_expires_copies_and_only_follows_the_original_exile_receipt() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &COPY_ABILITY)]);
    for target_is_token in [false, true] {
        let mut game = scenario(PLAYER);
        game.state.step = Step::Main2;
        pass_pair(&mut game);
        game.add_end_step_if_first();
        let turn = game.observe(PLAYER).turn;
        let source = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
        let recipient = token(&mut game, PLAYER, FOOD, false);
        let blueprint = if target_is_token {
            token(&mut game, PLAYER, Card::DrakeToken, true)
        } else {
            let id = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
            game.object_ref(id)
        };
        game.apply_temporary_copy(
            blueprint,
            renamed(named("Hill Giant"), "Tower Giant"),
            CopyDeadline::EndOfTurn,
        );
        start_activation(&mut game, source);
        choose_and_pay(&mut game, blueprint.card);
        settle(&mut game);
        let copy = view(&game, recipient.card);
        assert_eq!(copy.name, "Tower Giant");
        assert!(!copy.tapped);
        assert_eq!(game.observe(PLAYER).delayed_returns.len(), 1);
        let exile_ref = game.observe(PLAYER).delayed_returns[0].returning.object;
        if target_is_token {
            assert!(
                !game
                    .observe(PLAYER)
                    .objects
                    .iter()
                    .any(|o| o.id == blueprint.card)
            );
        } else {
            game.move_object(blueprint.card, Zone::Exile);
            assert_ne!(view(&game, blueprint.card).reference, exile_ref);
        }

        act(&mut game, Action::Pass);
        let checkpoint = game.snapshot();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        let mut indexed = game.clone();
        let revision = indexed.revision();
        indexed
            .apply_index(PLAYER.other(), revision, index)
            .unwrap();
        act(&mut game, Action::Pass);
        assert_eq!(indexed.snapshot(), game.snapshot());
        indexed.restore(&checkpoint).unwrap();
        indexed
            .apply_index(PLAYER.other(), indexed.revision(), index)
            .unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());

        let end = game.observe(PLAYER);
        assert_eq!((end.turn, end.step), (turn, Step::End));
        assert!(end.additional_end_steps.unwrap().current_is_additional);
        assert_eq!(view(&game, recipient.card).card, FOOD);
        assert_eq!(end.stack[0].kind, StackKind::DelayedReturn);
        settle(&mut game);
        if target_is_token {
            assert!(
                !game
                    .observe(PLAYER)
                    .objects
                    .iter()
                    .any(|o| o.id == blueprint.card)
            );
        } else {
            assert_eq!(view(&game, blueprint.card).zone, Zone::Exile);
        }
    }
}

#[test]
fn a_prevented_exile_still_supplies_the_copy_with_no_object_to_return() {
    struct ResetPrevention;
    impl Drop for ResetPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &COPY_ABILITY)]);
    let mut game = scenario(PLAYER);
    game.state.step = Step::Main2;
    let source = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let recipient = token(&mut game, PLAYER, FOOD, false);
    let creature = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let exact = game.object_ref(creature);
    let bounce = put(&mut game, PLAYER.other(), "Unsummon", Zone::Hand);
    start_activation(&mut game, source);
    choose_and_pay(&mut game, creature);

    let mut illegal = game.clone();
    act(&mut illegal, Action::Pass);
    cast(&mut illegal, bounce, Some(Target::Card(creature)));
    settle(&mut illegal);
    assert_eq!(view(&illegal, recipient.card).card, FOOD);
    assert!(illegal.observe(PLAYER).delayed_returns.is_empty());

    let _reset = ResetPrevention;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(exact)));
    settle(&mut game);
    assert_eq!(view(&game, creature).reference, exact);
    assert_eq!(view(&game, creature).zone, Zone::Battlefield);
    assert_eq!(view(&game, recipient.card).card, Card::GrizzlyBears);
    pass_pair(&mut game);
    assert_eq!(view(&game, recipient.card).card, FOOD);
    assert_eq!(game.observe(PLAYER).stack[0].kind, StackKind::DelayedReturn);
    settle(&mut game);
    assert_eq!(view(&game, creature).reference, exact);
}
