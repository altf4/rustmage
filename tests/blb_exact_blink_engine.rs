//! Blink effects keep their own entry results and exact delayed return links.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CREATURE_NONTOKEN: TargetPlan = one(TargetRule::Filtered {
    base: &TargetRule::Creature,
    predicate: &TargetPredicate::Nontoken,
});
const PARTING: CardRules = CardRules {
    effect: Effect::Spell(
        CREATURE_NONTOKEN,
        &[
            Instruction::GiveGift,
            Instruction::IfGift {
                promised: true,
                skip: 1,
            },
            Instruction::Exile,
            Instruction::IfGift {
                promised: false,
                skip: 1,
            },
            Instruction::BlinkUntilEndStepWithCounter {
                counter: CounterKind::PlusOne,
                actual_exile_only: false,
            },
        ],
    ),
    extras: &CardExtras {
        gift: Some(GiftSpec {
            kind: GiftKind::TappedFish,
            token: Some(Card(5317)),
            promised_targets: CREATURE_NONTOKEN,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const FLYING_RETURN: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::creature_you_control(),
                predicate: &TargetPredicate::WithoutKeyword(Keywords::FLYING),
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::BlinkUntilEndStepWithCounter {
            counter: CounterKind::Flying,
            actual_exile_only: true,
        }],
    }],
    ..CardRules::EMPTY
};
const PORTAL: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::CaptureEntryResult,
            Instruction::BlinkFollowingPublicMove { all_groups: false },
            Instruction::IfEntryResultSubtype {
                subtypes: &["Bird", "Frog", "Otter", "Rat"],
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn end_step(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
}

fn resolve_optional_entries(game: &mut Game) {
    for _ in 0..48 {
        if game.state.stack.is_empty() && game.state.pending == Pending::Priority {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::FinishTargets | Action::PutTrigger { .. } | Action::Pass
            )
        });
    }
    panic!("blink test exceeded its resolution bound");
}

#[test]
fn gift_branch_leaves_the_creature_exiled_and_other_branch_returns_exact_card_with_counter() {
    let _rules = fixture_rules::activate(&[(named("Disenchant"), &PARTING)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut base = scenario(player);
    let creature = put(&mut base, opponent, "Bear Cub", Zone::Battlefield);
    base.record_control(base.object_ref(creature), player, None);
    base.add_counter(creature, CounterKind::PlusOne, 2);
    let spell = put(&mut base, player, "Disenchant", Zone::Hand);
    act(&mut base, Action::AnnounceCast { card: spell });

    let mut promised = base.clone();
    act(
        &mut promised,
        Action::ChooseGift {
            beneficiary: Some(opponent),
        },
    );
    act(&mut promised, target(creature));
    pay(&mut promised);
    settle(&mut promised);
    assert!(
        promised
            .state
            .objects
            .iter()
            .enumerate()
            .any(|(id, object)| {
                object.zone == Zone::Battlefield
                    && object.is_token()
                    && object.controller == opponent
                    && object.tapped
                    && promised.has_subtype(id as CardId, "Fish")
            })
    );
    assert!(promised.observe(player).delayed_returns.is_empty());
    assert_eq!(promised.object_view(creature).zone, Zone::Exile);

    let decline = Action::ChooseGift { beneficiary: None };
    let mut indexed = base.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == decline)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut base, decline);
    assert_eq!(indexed.snapshot(), base.snapshot());
    act(&mut base, target(creature));
    pay(&mut base);
    settle(&mut base);
    let scheduled = base.observe(player).delayed_returns;
    assert_eq!(scheduled.len(), 1);
    assert_eq!(scheduled[0].returning.object, base.object_ref(creature));
    assert_eq!(
        scheduled[0].returning.counter,
        Some((CounterKind::PlusOne, 1))
    );
    assert_eq!(base.object_view(spell).zone, Zone::Graveyard);
    let before_end = base.snapshot();

    end_step(&mut base);
    assert!(matches!(
        base.state.stack.last().unwrap().kind,
        StackKind::DelayedReturn
    ));
    settle(&mut base);
    assert_eq!(base.object_view(creature).controller, opponent);
    assert_eq!(base.counter_count(creature, CounterKind::PlusOne), 1);

    base.restore(&before_end).unwrap();
    base.move_object(creature, Zone::Graveyard);
    base.move_object(creature, Zone::Exile);
    end_step(&mut base);
    settle(&mut base);
    assert_eq!(base.object_view(creature).zone, Zone::Exile);
}

#[test]
fn optional_nonflying_blink_returns_a_card_with_flying_and_has_a_token_noop_trigger() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &FLYING_RETURN)]);
    let player = Player::P0;
    let mut base = scenario(player);
    let creature = put(&mut base, player, "Bear Cub", Zone::Battlefield);
    let flying = put(&mut base, player, "Healer's Hawk", Zone::Battlefield);
    base.create_tokens(player, Card(5317), 1, false);
    let token = base
        .state
        .objects
        .iter()
        .position(|object| object.is_token())
        .unwrap() as CardId;
    let source = put(&mut base, player, "Hill Giant", Zone::Hand);
    cast(&mut base, source, None);
    pass_pair(&mut base);
    reject(&mut base, target(flying));
    let mut selecting_token = base.clone();
    let mut declining = base.clone();

    act(&mut declining, Action::FinishTargets);
    settle(&mut declining);
    assert!(declining.observe(player).delayed_returns.is_empty());

    act(&mut base, target(creature));
    settle(&mut base);
    base.move_object(source, Zone::Graveyard);
    end_step(&mut base);
    settle(&mut base);
    assert_eq!(base.counter_count(creature, CounterKind::Flying), 1);
    assert!(
        base.object_view(creature)
            .keywords
            .contains(Keywords::FLYING)
    );

    act(&mut selecting_token, target(token));
    settle(&mut selecting_token);
    end_step(&mut selecting_token);
    assert!(matches!(
        selecting_token.state.stack.last().unwrap().kind,
        StackKind::DelayedReturn
    ));
    settle(&mut selecting_token);
    assert!(!selecting_token.on_battlefield(token));
}

#[test]
fn immediate_blink_checks_returned_types_and_ignores_an_unrelated_linked_return() {
    let _rules = fixture_rules::activate(&[(named("Opt"), &PORTAL)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let hawk = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let fox = put(&mut game, player, "Werefox Bodyguard", Zone::Hand);
    cast(&mut game, fox, None);
    pass_pair(&mut game);
    act(&mut game, target(hawk));
    settle(&mut game);
    assert_eq!(game.object_view(hawk).zone, Zone::Exile);
    let original_fox = game.object_ref(fox);

    let first = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, first, Some(Target::Card(fox)));
    resolve_optional_entries(&mut game);
    assert_eq!(game.object_view(fox).zone, Zone::Battlefield);
    assert_ne!(game.object_ref(fox), original_fox);
    assert_eq!(game.object_view(hawk).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let original_hawk = game.object_ref(hawk);
    game.apply_temporary_copy(
        original_hawk,
        CopyRecipe::new(Card::GrizzlyBears),
        CopyDeadline::EndOfTurn,
    );
    assert!(!game.has_subtype(hawk, "Bird"));
    let second = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, second, Some(Target::Card(hawk)));
    settle(&mut game);
    assert_ne!(game.object_ref(hawk), original_hawk);
    assert!(game.has_subtype(hawk, "Bird"));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    game.create_tokens(player, Card::DrakeToken, 1, false);
    let token = game
        .state
        .objects
        .iter()
        .position(|object| object.is_token())
        .unwrap() as CardId;
    let token_ref = game.object_ref(token);
    game.apply_temporary_copy(
        token_ref,
        CopyRecipe::new(named("Healer's Hawk")),
        CopyDeadline::EndOfTurn,
    );
    assert!(game.has_subtype(token, "Bird"));
    let third = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, third, Some(Target::Card(token)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}
