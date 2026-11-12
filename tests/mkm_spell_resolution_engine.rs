//! Owned spell-resolution packets for grouped copies and reflexive effects.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const MULTIPLE_TARGET_COPIES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: usize::MAX,
        }),
        &[
            Instruction::TargetsExactlyX,
            Instruction::CreateCopiesOfEachTarget(AmountValue::ChosenX),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 3,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap_or_else(|| panic!("unknown fixture {name}"))
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn copies(game: &Game, player: Player, name: &str) -> Vec<ObjectView> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            if object.zone != Zone::Battlefield || object.controller != player {
                return None;
            }
            let view = game.object_view(id as CardId);
            (view.is_token() && view.card == named(name)).then_some(view)
        })
        .collect()
}

#[test]
fn each_legal_target_contributes_the_declared_number_to_one_token_batch() {
    let _rules = fixture_rules::activate(&[(named("Divination"), &MULTIPLE_TARGET_COPIES)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let doubler = put(&mut game, player, "Doubling Season", Zone::Battlefield);
        let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 12];
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(&mut game, Action::ChooseX { value: 3 });

        let mut zero = game.clone();
        act(&mut zero, Action::ChooseX { value: 0 });
        pay(&mut zero);
        settle(&mut zero);
        assert!(copies(&zero, player, "Doubling Season").is_empty());
        assert!(copies(&zero, player, "Forest").is_empty());

        let choice = Action::ChooseX { value: 2 };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == choice)
            .unwrap();
        let saved = game.snapshot();
        let revision = game.revision();
        let mut indexed = game.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, choice.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, choice);
        assert_eq!(game.snapshot(), indexed.snapshot());
        reject(&mut game, Action::FinishTargets);
        act(&mut game, target(doubler));
        reject(&mut game, target(doubler));
        act(&mut game, target(land));
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        assert_eq!(game.mana_payment_domain().unwrap().generic, 8);
        pay(&mut game);
        assert_eq!(game.state.stack.last().unwrap().mana_value(), 9);
        settle(&mut game);
        assert_eq!(copies(&game, player, "Forest").len(), 4);
        assert_eq!(copies(&game, player, "Doubling Season").len(), 4);
    }
}

#[test]
fn partial_target_failure_preserves_x_and_copies_only_the_live_copiable_values() {
    let _rules = fixture_rules::activate(&[(named("Divination"), &MULTIPLE_TARGET_COPIES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let copy = CopyRecipe::new(named("Grizzly Bears"));
    game.apply_temporary_copy(game.object_ref(creature), copy, CopyDeadline::EndOfTurn);
    game.add_counter(creature, CounterKind::PlusOne, 2);
    game.state.objects[creature as usize].tapped = true;
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseX { value: 2 });
    act(&mut game, target(creature));
    act(&mut game, target(land));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);

    let original_land = game.object_ref(land);
    game.move_object(land, Zone::Hand);
    game.move_object(land, Zone::Battlefield);
    assert_ne!(game.object_ref(land), original_land);
    let mut failed = game.clone();
    failed.move_object(creature, Zone::Hand);
    settle(&mut failed);
    assert!(copies(&failed, player, "Grizzly Bears").is_empty());

    settle(&mut game);
    assert!(copies(&game, player, "Forest").is_empty());
    let tokens = copies(&game, player, "Grizzly Bears");
    assert_eq!(tokens.len(), 2);
    for token in tokens {
        assert!(!token.tapped);
        assert_eq!(token.counters.get(CounterKind::PlusOne), 0);
        assert_eq!((token.power, token.toughness), (2, 2));
    }
}

const DISCARD_DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::ControllerHandSize),
                minimum: 2,
                skip: 2,
            },
            Instruction::May { skip: 1 },
            Instruction::DiscardExactlyReflexiveManaValue {
                amount: 2,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayerPermanents {
            players: PlayerSet::Each,
            kinds: 1,
            amount: AmountValue::DiscardedCardManaValue,
        }],
    }],
    ..CardRules::EMPTY
};

struct ClearPrevention;

impl Drop for ClearPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
    }
}

#[test]
fn exact_discards_capture_the_highest_value_for_a_separate_live_creature_sweep() {
    let _rules = fixture_rules::activate(&[(named("Divination"), &DISCARD_DAMAGE)]);
    let _clear = ClearPrevention;
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let peace = put(&mut game, opponent, "Rest in Peace", Zone::Battlefield);
        let own_creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let large_creature = put(
            &mut game,
            opponent,
            "Earthshaker Dreadmaw",
            Zone::Battlefield,
        );
        let opposing_hand = put(&mut game, opponent, "Hill Giant", Zone::Hand);
        let low = put(&mut game, player, "Grizzly Bears", Zone::Library);
        let high = put(&mut game, player, "Hill Giant", Zone::Library);
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        cast(&mut game, spell, None);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.hand_count(player), 2);
        assert_eq!(game.object_view(low).zone, Zone::Hand);
        assert_eq!(game.object_view(high).zone, Zone::Hand);
        assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));

        let mut declined = game.clone();
        act(&mut declined, Action::ChooseOptional { accept: false });
        assert!(declined.state.stack.is_empty());
        assert_eq!(declined.hand_count(player), 2);
        assert_eq!(declined.object_view(own_creature).damage, 0);

        act(&mut game, Action::ChooseOptional { accept: true });
        act(&mut game, Action::Discard { card: low });
        reject(&mut game, Action::Discard { card: low });
        reject(
            &mut game,
            Action::Discard {
                card: opposing_hand,
            },
        );
        reject(&mut game, Action::ChooseOptional { accept: false });
        assert_eq!(game.object_view(low).zone, Zone::Hand);
        assert!(game.observe(opponent).objects.iter().all(|o| o.id != low));

        let discard = Action::Discard { card: high };
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == discard)
            .unwrap();
        let saved = game.snapshot();
        let mut prevented = game.clone();
        {
            let _reset = ClearPrevention;
            zone_replacements::TEST_PREVENT_ZONE_MOVE
                .with(|slot| slot.set(Some(prevented.object_ref(high))));
            act(&mut prevented, discard.clone());
            act(
                &mut prevented,
                Action::ChooseZoneReplacement {
                    effect: ZoneReplacementKind::Prevent,
                },
            );
        }
        assert_eq!(prevented.object_view(high).zone, Zone::Hand);
        assert_eq!(prevented.object_view(low).zone, Zone::Exile);
        assert!(prevented.state.stack.is_empty());
        assert_eq!(prevented.object_view(own_creature).damage, 0);

        let mut indexed = game.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, discard.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, discard);
        assert_eq!(game.snapshot(), indexed.snapshot());

        assert_eq!(game.object_view(low).zone, Zone::Exile);
        assert_eq!(game.object_view(high).zone, Zone::Exile);
        assert_eq!(game.object_view(spell).zone, Zone::Exile);
        assert_eq!(
            game.state.players[player.index()].cards_discarded_this_turn,
            2
        );
        let stack = &game.state.stack;
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].kind, StackKind::TriggeredAbility { ability: 0 });
        assert_eq!(
            stack[0].event_object.as_ref().unwrap().facts,
            EventObjectFacts::DiscardedManaValue(4)
        );
        assert_eq!(game.object_view(own_creature).damage, 0);

        game.move_object(high, Zone::Hand);
        let later_creature = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        settle(&mut game);
        assert_eq!(game.object_view(own_creature).zone, Zone::Exile);
        assert_eq!(game.object_view(later_creature).zone, Zone::Exile);
        assert_eq!(game.object_view(large_creature).damage, 4);
        assert_eq!(game.object_view(peace).zone, Zone::Battlefield);
        assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE);
    }
}

#[test]
fn exact_optional_discard_is_unavailable_with_an_insufficient_hand() {
    const DRAW_LOCK: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CantDrawCards],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (named("Divination"), &DISCARD_DAMAGE),
        (named("Short Bow"), &DRAW_LOCK),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let only = put(&mut game, player, "Hill Giant", Zone::Hand);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
    assert_eq!(game.object_view(only).zone, Zone::Hand);
    assert_eq!(
        game.state.players[player.index()].cards_discarded_this_turn,
        0
    );
}
