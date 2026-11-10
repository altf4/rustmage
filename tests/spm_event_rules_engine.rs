//! Frozen play origins and actual combat events.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ORIGINS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayFromOutsideOwnHand(
                &TriggerCondition::SpellCastOrLandPlayed,
            ),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::CastFromExile(&TriggerCondition::SpellCastOrLandPlayed),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};

const LAND_RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::ReturnToBattlefield],
    ),
    ..CardRules::EMPTY
};

fn order_triggers(game: &mut Game) {
    for _ in 0..4 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            return;
        }
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    panic!("expected at most four simultaneous triggers");
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    act(game, Action::Pass);
    act(game, Action::Pass);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn exile_and_offhand_observers_distinguish_land_plays_casts_and_direct_entries() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &ORIGINS),
        (Card::Divination, &LAND_RETURN),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let hand = put(&mut game, player, "Forest", Zone::Hand);
        let exile = put(&mut game, player, "Mountain", Zone::Exile);
        let grave = put(&mut game, player, "Island", Zone::Graveyard);
        let shock = put(&mut game, player, "Shock", Zone::Exile);
        game.grant_exile_play_permission(player, game.object_ref(exile), false);
        game.grant_exile_play_permission(player, game.object_ref(shock), false);

        let spell = put(&mut game, player, "Divination", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(grave)));
        settle(&mut game);
        assert_eq!(game.object_view(grave).zone, Zone::Battlefield);
        assert!(
            !game.state.players[player.index()]
                .turn_facts
                .played_from_outside_own_hand
        );
        assert_eq!(game.state.players[player.index()].life, 20);

        let before = game.snapshot();
        let mut indexed = game.clone();
        let play = Action::PlayLand { card: exile };
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == play)
            .unwrap();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut game, play);
        assert_eq!(game.snapshot(), indexed.snapshot());

        indexed.restore(&before).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, Action::PlayLand { card: hand });
        settle(&mut indexed);
        assert_eq!(indexed.state.players[player.index()].life, 20);
        assert!(
            !indexed.state.players[player.index()]
                .turn_facts
                .played_from_outside_own_hand
        );

        order_triggers(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 21);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        assert!(
            game.state.players[player.index()]
                .turn_facts
                .played_from_outside_own_hand
        );

        act(&mut game, Action::AnnounceExile { card: shock });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        pay(&mut game);
        order_triggers(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 22);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
        assert_eq!(game.state.players[player.other().index()].life, 18);
    }
}

const COMBAT_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ControlledSubtypeAttackAtLeast {
                subtype: "Bird",
                minimum: 2,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::ControlledCreatureAttacksWithKeyword(Keywords::FLYING),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn declaration_counts_qualified_subtypes_once_and_live_keywords_per_attacker() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &COMBAT_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let first = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let second = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);

    let mut one = game.clone();
    declare(&mut one, &[first, bear]);
    settle(&mut one);
    assert_eq!(one.state.players[player.index()].life, 21);

    declare(&mut game, &[first, second, bear]);
    order_triggers(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 25);
}

const CAST_VALUE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::CastManaValueAtLeastFourThisTurn,
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn a_real_x_cast_sets_the_threshold_before_an_observer_enters() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &CAST_VALUE_OBSERVER)]);
    let player = Player::P0;
    for x in [1, 2] {
        let mut game = scenario(player);
        let finale = put(&mut game, player, "Finale of Revelation", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: finale });
        act(&mut game, Action::ChooseX { value: x });
        pay(&mut game);
        assert_eq!(
            game.state.players[player.index()]
                .turn_facts
                .cast_mana_value_at_least_four,
            x == 2
        );
        assert!(
            !game.state.players[player.other().index()]
                .turn_facts
                .cast_mana_value_at_least_four
        );
        settle(&mut game);
        let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        declare(&mut game, &[attacker]);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].life,
            if x == 2 { 23 } else { 20 }
        );
    }
}

const DAMAGE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::DamageWithController {
            damage: 2,
            controller_damage: 2,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SourceDealtDamageAtLeast(4),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn source_threshold_counts_actual_simultaneous_damage_after_prevention() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &DAMAGE_OBSERVER)]);
    let player = Player::P0;
    for shield in [false, true] {
        let mut game = scenario(player);
        let dealer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let recipient = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        if shield {
            game.add_counter(recipient, CounterKind::Shield, 1);
        }
        act(
            &mut game,
            Action::AnnounceActivate {
                card: dealer,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(recipient),
            },
        );
        if game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::PayCosts { .. }))
        {
            pay(&mut game);
        }
        settle(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 18);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(!shield)
        );
        assert_eq!(
            game.object_view(recipient).zone,
            if shield {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
}
