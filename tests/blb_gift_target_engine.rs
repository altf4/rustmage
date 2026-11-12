//! Gift resolution and first target acquisition for exact permanents.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const GIFT_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::GiftGiven,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const FIRST_TARGET: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FirstTargetedByController,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};

const MULTIPLIER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledLegendaryCreature,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};

fn settle_triggers(game: &mut Game) {
    for _ in 0..56 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::PutTrigger { .. } | Action::Pass)
        });
    }
    panic!("gift and target triggers did not settle");
}

#[test]
fn giving_a_gift_is_an_owned_resolution_event_even_when_its_payload_changes() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GIFT_OBSERVER)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        put(&mut game, other, "Doubling Season", Zone::Battlefield);
        let victim = put(&mut game, other, "Hill Giant", Zone::Battlefield);
        let removal = put(&mut game, player, "Nocturnal Hunger", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: removal });
        let gift = Action::ChooseGift {
            beneficiary: Some(other),
        };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == gift)
            .unwrap();
        let saved = game.snapshot();
        let mut raw = game.clone();
        act(&mut raw, gift);
        game.apply_index(player, game.revision(), index).unwrap();
        let target = Action::ChooseTarget {
            target: Target::Card(victim),
        };
        for branch in [&mut raw, &mut game] {
            act(branch, target.clone());
            pay(branch);
            assert_eq!(
                branch.state.players[player.index()].cards_drawn_this_turn,
                0
            );
        }

        let mut stale = raw.clone();
        stale.move_object(victim, Zone::Exile);
        settle_triggers(&mut stale);
        assert_eq!(stale.state.players[player.index()].cards_drawn_this_turn, 0);

        for branch in [&mut raw, &mut game] {
            settle_triggers(branch);
            assert_eq!(
                branch.state.players[player.index()].cards_drawn_this_turn,
                1
            );
            assert_eq!(branch.state.players[other.index()].cards_drawn_this_turn, 0);
            assert_eq!(branch.object_view(victim).zone, Zone::Graveyard);
            assert_eq!(
                branch
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|&(id, object)| {
                        object.zone == Zone::Battlefield
                            && object.controller == other
                            && object.is_token()
                            && branch.has_subtype(id as CardId, "Food")
                    })
                    .count(),
                2
            );
        }
        assert_eq!(raw.snapshot(), game.snapshot());

        game.restore(&saved).unwrap();
        act(&mut game, Action::ChooseGift { beneficiary: None });
        act(&mut game, target);
        pay(&mut game);
        settle_triggers(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    }
}

#[test]
fn permanent_gift_resolves_separately_after_the_source_departs() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GIFT_OBSERVER)]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let source = put(&mut game, player, "Scrapshooter", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: source });
    act(
        &mut game,
        Action::ChooseGift {
            beneficiary: Some(other),
        },
    );
    pay(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    assert_eq!(game.state.players[other.index()].cards_drawn_this_turn, 0);
    game.move_object(source, Zone::Exile);
    settle_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.state.players[other.index()].cards_drawn_this_turn, 1);
}

#[test]
fn first_own_target_event_can_trigger_an_additional_time() {
    let legendary = Card::from_name("Wylie Duke, Atiin Hero").unwrap();
    let provider = Card::from_name("Doubling Season").unwrap();
    let _rules = fixture_rules::activate(&[(legendary, &FIRST_TARGET), (provider, &MULTIPLIER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    spawn(&mut game, player, provider, Zone::Battlefield);
    let source = spawn(&mut game, player, legendary, Zone::Battlefield);

    let opposing = put(&mut game, player.other(), "Giant Growth", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, opposing, Some(Target::Card(source)));
    settle_triggers(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);

    for _ in 0..2 {
        let spell = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(source)));
        settle_triggers(&mut game);
        assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
    }
}

#[test]
fn target_history_survives_acquiring_an_ability_and_resets_on_reentry() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FIRST_TARGET)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let old = game.object_ref(creature);
    let spell = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle_triggers(&mut game);

    let recipe = CopyRecipe::new(Card::GrizzlyBears);
    game.apply_temporary_copy(old, recipe, CopyDeadline::EndOfTurn);
    let spell = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle_triggers(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 0);

    game.move_object(creature, Zone::Exile);
    game.move_object(creature, Zone::Battlefield);
    let new = game.object_ref(creature);
    assert_ne!(old, new);
    game.apply_temporary_copy(new, recipe, CopyDeadline::EndOfTurn);
    let spell = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle_triggers(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
}

#[test]
fn offspring_reads_live_or_last_known_copiable_values_and_sets_the_base_to_one_one() {
    const OFFSPRING: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfOffspring,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Source,
                amount: 1,
                exceptions: CopyExceptions {
                    base: Some((1, 1)),
                    ..CopyExceptions::NONE
                },
            }],
        }],
        extras: &CardExtras {
            offspring_cost: Some(ManaCost {
                colored: [0; 6],
                generic: 1,
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OFFSPRING)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let parent = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: parent });
    act(&mut game, Action::ChooseOffspring { pay: true });
    pay(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(parent).zone, Zone::Battlefield);

    let blueprint = Card::from_name("Serra Angel").unwrap();
    let parent_reference = game.object_ref(parent);
    game.apply_temporary_copy(
        parent_reference,
        CopyRecipe::new(blueprint),
        CopyDeadline::EndOfTurn,
    );
    game.add_counter(parent, CounterKind::PlusOne, 3);

    let mut departed = game.clone();
    departed.move_object(parent, Zone::Exile);
    for branch in [&mut game, &mut departed] {
        settle_triggers(branch);
        let child = branch
            .state
            .objects
            .iter()
            .position(|object| object.is_copy_token() && object.zone == Zone::Battlefield)
            .expect("offspring copy") as CardId;
        let view = branch.object_view(child);
        assert_eq!(view.card, blueprint);
        assert_eq!(view.controller, player);
        assert_eq!(stats(branch, child), (1, 1));
        assert!(view.keywords.contains(Keywords::FLYING));
        assert!(view.keywords.contains(Keywords::VIGILANCE));
        assert_eq!(branch.counter_count(child, CounterKind::PlusOne), 0);
    }
}
