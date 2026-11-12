//! Exact damage, cast, mill, and search result continuations.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const FIRE_BONUS: ReplacementSpec =
    ReplacementSpec::ControlledDamageToOpponentPlusSourceCounters(CounterKind::Fire);
const FIRE_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, FIRE_BONUS)],
}];
const FIREPOWER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: FIRE_GROUPS,
    extras: &CardExtras {
        static_ability_groups: FIRE_GROUPS,
        replacement_effects: &[FIRE_BONUS],
        x_symbols: 1,
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::ChosenX(CounterKind::Fire),
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ZERO_DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[Instruction::Damage(0)],
    ),
    ..CardRules::EMPTY
};
const ABILITY: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    exhaust: false,
    once_per_turn: false,
    sorcery: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
};
const LESSON_COPY: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[ActivatedAbility {
        program: &[Instruction::CopyNextSpell { ability: 0 }],
        ..ABILITY
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedSpellCastSubtype("Lesson"),
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCapturedSpell {
            count: SpellCopyCount::One,
            retarget: true,
        }],
    }],
    ..CardRules::EMPTY
};
const MILL: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(1)),
            Instruction::RecoverMilled(RecoveryFilter::Land),
            Instruction::IfAmountAtLeast {
                value: AmountValue::MilledSubtypeCount("Lesson"),
                minimum: 1,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
        ..ABILITY
    }],
    ..CardRules::EMPTY
};
const FOOD_SEARCH: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FoodsSacrificed,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 3 },
            Instruction::SearchWithoutShuffle(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            }),
            Instruction::PutSourceOnLibraryBottom,
            Instruction::ShuffleAfterSearches,
        ],
    }],
    ..CardRules::EMPTY
};
const EXILE_AND_CAST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileUntilNonlandCastBelowOrHand(
            AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 2,
                exclude: 0,
                subtype: Some("Mountain"),
                basic: false,
                other: false,
                cards_only: false,
            })),
        )],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn activate(game: &mut Game, source: CardId, index: u16) {
    choose(
        game,
        |action| matches!(action, Action::Activate { card, ability, .. } if *card == source && *ability == index),
    );
}

fn cast_firebending_lesson(game: &mut Game, card: CardId, target: CardId) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseKicker { pay: false });
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(game);
}

#[test]
fn chosen_x_is_placed_in_the_named_counter_kind_and_can_be_replaced() {
    let _rules = fixture_rules::activate(&[(named("Impact Tremors"), &FIREPOWER)]);
    let p = Player::P1;
    let mut game = scenario(p);
    put(&mut game, p, "Doubling Season", Zone::Battlefield);
    let card = put(&mut game, p, "Impact Tremors", Zone::Hand);
    let mut noncast = game.clone();
    noncast.move_object(card, Zone::Battlefield);
    assert_eq!(noncast.counter_count(card, CounterKind::Fire), 0);

    act(&mut game, Action::AnnounceCast { card });
    act(&mut game, Action::ChooseX { value: 3 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(card, CounterKind::Fire), 6);
    assert_eq!(game.counter_count(card, CounterKind::PlusOne), 0);
}

#[test]
fn a_damage_replacement_uses_live_counters_for_opposing_recipients() {
    let _rules = fixture_rules::activate(&[
        (named("Impact Tremors"), &FIREPOWER),
        (named("Shock"), &ZERO_DAMAGE),
    ]);
    let p = Player::P0;
    let mut game = scenario(p);
    let provider = put(&mut game, p, "Impact Tremors", Zone::Battlefield);
    game.add_counter(provider, CounterKind::Fire, 2);
    let bolt = put(&mut game, p, "Lightning Strike", Zone::Hand);
    cast(&mut game, bolt, Some(Target::Player(p.other())));
    let mut removed = game.clone();
    removed.move_object(provider, Zone::Exile);
    settle(&mut removed);
    assert_eq!(removed.state.players[p.other().index()].life, 17);

    game.add_counter(provider, CounterKind::Fire, 1);
    settle(&mut game);
    assert_eq!(game.state.players[p.other().index()].life, 14);
    let self_bolt = put(&mut game, p, "Lightning Strike", Zone::Hand);
    cast(&mut game, self_bolt, Some(Target::Player(p)));
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 17);
    let zero = put(&mut game, p, "Shock", Zone::Hand);
    cast(&mut game, zero, Some(Target::Player(p.other())));
    settle(&mut game);
    assert_eq!(game.state.players[p.other().index()].life, 14);

    let creature = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(creature, CounterKind::PlusOne, 5);
    let bolt = put(&mut game, p, "Lightning Strike", Zone::Hand);
    cast(&mut game, bolt, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.state.objects[creature as usize].damage, 6);
}

#[test]
fn damage_bonus_does_not_reduce_the_required_trample_assignment() {
    let _rules = fixture_rules::activate(&[(named("Impact Tremors"), &FIREPOWER)]);
    let p = Player::P1;
    let mut game = scenario(p);
    let provider = put(&mut game, p, "Impact Tremors", Zone::Battlefield);
    game.add_counter(provider, CounterKind::Fire, 3);
    let attacker = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.add_counter(attacker, CounterKind::PlusOne, 4);
    game.state.objects[attacker as usize].gained_keywords = Keywords::TRAMPLE;
    let blocker = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    let range = game.damage_assignment_range().unwrap();
    assert_eq!(range.target, Target::Card(blocker));
    assert_eq!(range.minimum, 3);
    act(
        &mut game,
        Action::AssignDamage {
            target: range.target,
            amount: 3,
        },
    );
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(game.state.objects[attacker as usize].damage, 3);
    assert_eq!(game.state.players[p.other().index()].life, 14);
}

#[test]
fn a_delayed_lesson_copy_survives_departure_and_consumes_only_the_first_cast() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &LESSON_COPY)]);
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    activate(&mut game, source, 0);
    settle(&mut game);
    assert_eq!(game.delayed_spell_copy_view().len(), 1);
    let mut expired = game.clone();
    expired.state.step = Step::End;
    pass_pair(&mut expired);
    assert!(expired.delayed_spell_copy_view().is_empty());

    game.move_object(source, Zone::Graveyard);
    let ordinary = put(&mut game, p, "Lightning Strike", Zone::Hand);
    cast(&mut game, ordinary, Some(Target::Player(p.other())));
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.delayed_spell_copy_view().len(), 1);

    let left = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let right = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let lesson = put(&mut game, p, "Firebending Lesson", Zone::Hand);
    cast_firebending_lesson(&mut game, lesson, left);
    assert_eq!(game.state.stack.len(), 2);
    assert!(game.delayed_spell_copy_view().is_empty());

    let mut countered = game.clone();
    act(&mut countered, Action::Pass);
    let negate = put(&mut countered, p.other(), "Negate", Zone::Hand);
    cast(&mut countered, negate, Some(Target::Spell(lesson)));
    pass_pair(&mut countered);
    assert_eq!(countered.object_view(lesson).zone, Zone::Graveyard);
    assert_eq!(countered.state.stack.len(), 1);

    pass_pair(&mut game);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(right),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    assert!(game.state.stack.last().unwrap().is_copy);
    settle(&mut game);
    assert_eq!(game.state.objects[left as usize].damage, 2);
    assert_eq!(game.state.objects[right as usize].damage, 2);
    let later = put(&mut game, p, "Firebending Lesson", Zone::Hand);
    cast_firebending_lesson(&mut game, later, left);
    assert_eq!(game.state.stack.len(), 1);
}

#[test]
fn mill_subtype_receipts_follow_public_exile_and_only_the_current_mill() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &MILL)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    put(&mut game, p, "Firebending Lesson", Zone::Graveyard);
    let mut land = game.clone();
    let top = put(&mut land, p, "Island", Zone::Library);
    activate(&mut land, source, 0);
    pass_pair(&mut land);
    let recovered = land.object_ref(top);
    act(
        &mut land,
        Action::RecoverMilled {
            object: Some(recovered),
        },
    );
    settle(&mut land);
    assert_eq!(land.object_view(top).zone, Zone::Hand);
    assert_eq!(land.state.players[p.index()].life, 20);

    let mut ordinary = game.clone();
    put(&mut ordinary, p, "Hill Giant", Zone::Library);
    activate(&mut ordinary, source, 0);
    settle(&mut ordinary);
    assert_eq!(ordinary.state.players[p.index()].life, 20);

    put(&mut game, p.other(), "Rest in Peace", Zone::Battlefield);
    let lesson = put(&mut game, p, "Firebending Lesson", Zone::Library);
    activate(&mut game, source, 0);
    settle(&mut game);
    assert_eq!(game.object_view(lesson).zone, Zone::Exile);
    assert_eq!(game.state.players[p.index()].life, 22);
}

#[test]
fn an_accepted_food_search_bottoms_the_exact_live_source_and_shuffles_the_searcher() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &FOOD_SEARCH)]);
    let p = Player::P1;
    let owner = p.other();
    let mut game = scenario(p);
    let source = put(&mut game, owner, "Grizzly Bears", Zone::Battlefield);
    game.change_controller(game.object_ref(source), p);
    let old_top = game.object_ref(game.top_library_card(owner).unwrap());
    game.remember_library_top(owner, old_top);
    let gingerbrute = put(&mut game, p, "Gingerbrute", Zone::Battlefield);
    let land = put(&mut game, p, "Island", Zone::Library);
    let rng = game.state.rng;
    activate(&mut game, gingerbrute, 1);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.object_view(source).zone, Zone::Battlefield);
    assert_eq!(decline.state.rng, rng);

    let mut empty = game.clone();
    act(&mut empty, Action::ChooseOptional { accept: true });
    act(&mut empty, Action::FinishSearch);
    settle(&mut empty);
    assert_eq!(
        empty.state.players[owner.index()].library.first(),
        Some(&source)
    );
    assert_ne!(empty.state.rng, rng);

    let mut returned = game.clone();
    returned.move_object(source, Zone::Exile);
    returned.move_object(source, Zone::Battlefield);
    act(&mut returned, Action::ChooseOptional { accept: true });
    act(&mut returned, Action::FinishSearch);
    settle(&mut returned);
    assert_eq!(returned.object_view(source).zone, Zone::Battlefield);
    assert_ne!(returned.state.rng, rng);

    act(&mut game, Action::ChooseOptional { accept: true });
    act(&mut game, Action::ChooseSearch { card: land });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(
        game.state.players[owner.index()].library.first(),
        Some(&source)
    );
    assert_eq!(game.public_library_top(owner), Some(old_top));
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    assert_ne!(game.state.rng, rng);
}

#[test]
fn a_strict_cast_limit_uses_the_prospective_adventure_face_and_falls_back_to_hand() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &EXILE_AND_CAST)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Opt", Zone::Hand);
    let found = put(&mut game, p, "Virtue of Persistence", Zone::Library);
    let leading_land = put(&mut game, p, "Forest", Zone::Library);
    let target = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let mut no_mountains = game.clone();
    cast(&mut no_mountains, source, None);
    settle(&mut no_mountains);
    assert_eq!(no_mountains.object_view(found).zone, Zone::Hand);
    assert_eq!(no_mountains.object_view(leading_land).zone, Zone::Exile);

    for _ in 0..2 {
        put(&mut game, p, "Mountain", Zone::Battlefield);
    }
    let mut equal = game.clone();
    cast(&mut equal, source, None);
    pass_pair(&mut equal);
    let adventure = |game: &Game| Action::CastExiledDuringResolution {
        object: game.object_ref(found),
        face: CastFace::Adventure,
    };
    let denied = adventure(&equal);
    reject(&mut equal, denied);
    act(&mut equal, Action::FinishResolutionCasts);
    settle(&mut equal);
    assert_eq!(equal.object_view(found).zone, Zone::Hand);

    put(&mut game, p, "Mountain", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    let rejected = Action::CastExiledDuringResolution {
        object: game.object_ref(found),
        face: CastFace::Normal,
    };
    reject(&mut game, rejected);
    let offered = adventure(&game);
    act(&mut game, offered);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(
        game.state
            .stack
            .last()
            .unwrap()
            .source()
            .unwrap()
            .card
            .name(),
        "Locthwain Scorn"
    );
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    assert_eq!(game.object_view(found).zone, Zone::Exile);
    assert_eq!(game.object_view(leading_land).zone, Zone::Exile);
}
