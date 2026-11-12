//! Power-qualified trigger multiplicity and block legality.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, generic, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROVIDER: Card = Card::from_catalogue_id(1033); // Delney, Streetwise Lookout
const PROVIDER_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::ControlledCreaturesCannotBeBlockedByPower {
                    attacker_maximum: 2,
                    blocker_minimum: 3,
                },
            ],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::ControlledCreaturePowerAtMost(2),
                condition: StatValue::Fixed(1),
            }],
        },
    ],
    ..CardRules::EMPTY
};
const ENTERS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const ENTERS_LARGE: CardRules = CardRules {
    enters: InitialCounters {
        plus_one: 1,
        ..ENTERS.enters
    },
    ..ENTERS
};
const DIES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const WARD: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: generic(2),
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn order_trigger(game: &mut Game) {
    choose(game, |action| matches!(action, Action::PutTrigger { .. }));
}

fn order_triggers(game: &mut Game) {
    for _ in 0..8 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            return;
        }
        order_trigger(game);
    }
    panic!("trigger order did not finish");
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

#[test]
fn extra_entry_trigger_targets_are_independent_and_power_is_frozen_at_entry() {
    let entrant = Card::from_name("Novice Inspector").unwrap();
    let large_entrant = Card::from_name("Savannah Lions").unwrap();
    let _rules = fixture_rules::activate(&[
        (PROVIDER, &PROVIDER_RULES),
        (entrant, &ENTERS),
        (large_entrant, &ENTERS_LARGE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, PROVIDER, Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let adversary = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let card = spawn(&mut game, player, entrant, Zone::Hand);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    let mut large = game.clone();
    let card_large = spawn(&mut large, player, large_entrant, Zone::Hand);
    cast(&mut large, card_large, None);
    pass_pair(&mut large);
    assert_eq!(stats(&large, card_large).0, 3);
    assert!(large.state.ordering_triggers.is_empty());
    act(&mut large, target(first));
    settle(&mut large);
    assert_eq!(large.counter_count(first, CounterKind::PlusOne), 1);

    cast(&mut game, card, None);
    pass_pair(&mut game);
    let [a, b] = &game.state.ordering_triggers[..] else {
        panic!("a qualifying entry should queue two triggers");
    };
    assert_ne!(a.id, b.id);
    assert!(!a.is_copy && !b.is_copy);
    assert_eq!(a.source.unwrap().object, game.object_ref(card));
    assert_eq!(b.source.unwrap().object, game.object_ref(card));
    order_trigger(&mut game);
    reject(&mut game, target(adversary));
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == target(first))
        .unwrap();
    let checkpoint = game.snapshot();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, target(first));
    assert_eq!(game.snapshot(), indexed.snapshot());

    act(&mut game, target(second));
    cast(&mut game, growth, Some(Target::Card(card)));
    settle(&mut game);
    assert_eq!(stats(&game, card).0, 5);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 1);

    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.apply_index(player, revision, index).is_err());
    act(&mut indexed, target(second));
    act(&mut indexed, target(second));
    settle(&mut indexed);
    assert_eq!(indexed.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(indexed.counter_count(second, CounterKind::PlusOne), 2);
}

#[test]
fn simultaneous_deaths_use_last_power_and_controller_even_when_provider_dies() {
    let _rules =
        fixture_rules::activate(&[(PROVIDER, &PROVIDER_RULES), (Card::GrizzlyBears, &DIES)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let provider = spawn(&mut game, player, PROVIDER, Zone::Battlefield);
    let large = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let small = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = spawn(&mut game, opponent, Card::GrizzlyBears, Zone::Battlefield);
    game.add_counter(large, CounterKind::PlusOne, 1);
    let wrath = put(&mut game, player, "Day of Judgment", Zone::Hand);

    cast(&mut game, wrath, None);
    pass_pair(&mut game);
    for id in [provider, large, small, enemy] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    order_triggers(&mut game);
    assert_eq!(game.state.stack.len(), 4);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[opponent.index()].life, 21);
}

#[test]
fn a_low_power_creatures_ward_requires_two_independent_payments() {
    let _rules =
        fixture_rules::activate(&[(PROVIDER, &PROVIDER_RULES), (Card::GrizzlyBears, &WARD)]);
    let caster = Player::P0;
    let defender = caster.other();
    let mut game = scenario(caster);
    spawn(&mut game, defender, PROVIDER, Zone::Battlefield);
    let creature = spawn(&mut game, defender, Card::GrizzlyBears, Zone::Battlefield);
    let shock = put(&mut game, caster, "Shock", Zone::Hand);
    let mut large = game.clone();
    large.add_counter(creature, CounterKind::PlusOne, 1);
    cast(&mut large, shock, Some(Target::Card(creature)));
    assert!(large.state.ordering_triggers.is_empty());
    assert_eq!(large.state.stack.len(), 2);

    cast(&mut game, shock, Some(Target::Card(creature)));
    order_triggers(&mut game);
    let [spell, first_ward, second_ward] = &game.state.stack[..] else {
        panic!("expected the spell and both ward triggers");
    };
    for ward in [first_ward, second_ward] {
        assert_eq!(ward.related_stack.unwrap().id, spell.id);
        assert!(!ward.is_copy);
    }
    let payment = Action::PayResolutionMana {
        payment: [0, 0, 0, 0, 0, 2],
    };
    pass_pair(&mut game);
    act(&mut game, payment.clone());
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(caster));
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert_eq!(declined.object_view(creature).damage, 0);

    act(&mut game, payment);
    settle(&mut game);
    assert_eq!(game.object_view(creature).damage, 2);
    assert_eq!(game.state.players[caster.index()].mana[5], 4);
}

#[test]
fn block_declarations_use_live_power_on_both_sides_and_keep_existing_blocks() {
    let _rules = fixture_rules::activate(&[(PROVIDER, &PROVIDER_RULES)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    spawn(&mut game, player, PROVIDER, Zone::Battlefield);
    let small = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let large = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let small_blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let large_blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let attacker_forest = put(&mut game, player, "Forest", Zone::Battlefield);
    let defender_forest = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let attacker_growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    let defender_growth = put(&mut game, opponent, "Giant Growth", Zone::Hand);
    game.add_counter(large, CounterKind::PlusOne, 1);
    game.add_counter(large_blocker, CounterKind::PlusOne, 1);
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.state.pending, Pending::Attackers);
    act(&mut game, Action::Attack { card: small });
    act(&mut game, Action::Attack { card: large });
    act(&mut game, Action::FinishAttackers);

    let forbidden = Action::Block {
        blocker: large_blocker,
        attacker: small,
    };
    let mut raised = game.clone();
    act(
        &mut raised,
        Action::TapMana {
            card: attacker_forest,
        },
    );
    cast(&mut raised, attacker_growth, Some(Target::Card(small)));
    settle(&mut raised);
    pass_pair(&mut raised);
    assert_eq!(stats(&raised, small).0, 5);
    act(&mut raised, forbidden.clone());
    act(&mut raised, Action::FinishBlockers);

    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::Blockers);
    reject(&mut game, forbidden);
    assert!(game.legal_actions().contains(&Action::Block {
        blocker: large_blocker,
        attacker: large,
    }));
    act(
        &mut game,
        Action::Block {
            blocker: small_blocker,
            attacker: small,
        },
    );
    act(&mut game, Action::FinishBlockers);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));
    act(
        &mut game,
        Action::TapMana {
            card: defender_forest,
        },
    );
    cast(
        &mut game,
        defender_growth,
        Some(Target::Card(small_blocker)),
    );
    settle(&mut game);
    assert_eq!(stats(&game, small_blocker).0, 5);
    assert!(game.state.blocks.contains(&(small_blocker, small)));
    assert!(
        game.state
            .attackers
            .iter()
            .any(|attack| attack.card == small && attack.blocked)
    );
}
