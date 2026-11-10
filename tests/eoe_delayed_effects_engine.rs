//! Exact delayed cleanup, grant dispatch and frozen activation receipts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::engine::temporary_tokens::TokenCleanup;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn advance_until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..120 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("scenario did not reach its stop condition");
}

fn order_trigger(game: &mut Game) {
    if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

const NEXT_TURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensThenCleanup {
            token: Card::from_catalogue_id(5309),
            amount: 1,
            attacking: false,
            cleanup: TokenCleanup::NextOwnTurnEndStepSacrifice,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn temporary_token_waits_for_its_creators_next_own_turn() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &NEXT_TURN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, spell, None);
    resolve_top(&mut game);
    let token = spell + 1;
    assert_eq!(game.object_view(token).zone, Zone::Battlefield);
    let turn = game.state.turn;
    advance_until(&mut game, |g| g.state.turn == turn + 1);
    assert_eq!(game.object_view(token).zone, Zone::Battlefield);
    advance_until(&mut game, |g| g.state.turn == turn + 2);
    assert_eq!(game.object_view(token).zone, Zone::Battlefield);
    advance_until(&mut game, |g| g.state.turn == turn + 3);
    assert_ne!(game.object_view(token).zone, Zone::Battlefield);
}

const COPIES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::CreateHastyCopiesWithDelayedSacrifice(
            AmountValue::Fixed(2),
        )],
    ),
    ..CardRules::EMPTY
};

#[test]
fn hasty_copies_have_one_exact_cleanup_batch_and_leave_the_original() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &COPIES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let original = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(original)));
    resolve_top(&mut game);
    for token in [spell + 1, spell + 2] {
        let view = game.object_view(token);
        assert!(view.is_token());
        assert!(view.keywords.contains(Keywords::HASTE));
        assert_eq!(view.card, game.object_view(original).card);
    }
    let turn = game.state.turn;
    advance_until(&mut game, |g| g.state.turn > turn);
    for token in [spell + 1, spell + 2] {
        assert_ne!(game.object_view(token).zone, Zone::Battlefield);
    }
    assert_eq!(game.object_view(original).zone, Zone::Battlefield);
}

const COUNTERS: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[Instruction::PutCountersThenScheduleEndStep {
            subject: Subject::Targets,
            kind: CounterKind::Charge,
            amount: 10,
            ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::RemoveDelayedObjectCounters {
            kind: CounterKind::Charge,
            amount: 10,
        }],
    }],
    ..CardRules::EMPTY
};
const COUNTER_STOP: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(16)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn counter_cleanup_requires_actual_placement_and_tracks_the_exact_recipient() {
    let stop = Card::from_name("Swiftfoot Boots").unwrap();
    let _rules = fixture_rules::activate(&[(Card::Divination, &COUNTERS), (stop, &COUNTER_STOP)]);
    let player = Player::P0;
    for (blocked, blinked, expected) in [(false, false, 2), (false, true, 12), (true, false, 12)] {
        let mut game = scenario(player);
        if blocked {
            spawn(&mut game, player, stop, Zone::Battlefield);
        }
        let artifact = put(&mut game, player, "Nutrient Block", Zone::Battlefield);
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(artifact)));
        resolve_top(&mut game);
        assert_eq!(
            game.counter_count(artifact, CounterKind::Charge),
            if blocked { 0 } else { 10 }
        );
        if blinked {
            let previous = game.object_ref(artifact);
            game.move_object(artifact, Zone::Exile);
            game.move_object(artifact, Zone::Battlefield);
            assert!(game.chosen_source_objects().0.contains(&previous));
        }
        game.set_counters(artifact, Counters::of(CounterKind::Charge, 12));
        let turn = game.state.turn;
        advance_until(&mut game, |g| g.state.turn > turn);
        assert_eq!(game.counter_count(artifact, CounterKind::Charge), expected);
    }
}

const GRANT: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::GrantUntilEndTurn { ability: 0 },
            Instruction::MayTargetController { skip: 1 },
            Instruction::Damage(1),
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SelfDamaged),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::DamageReceivedAmount),
            Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::DamageReceivedAmount),
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn temporary_damage_grant_observes_one_lethal_batch_for_the_recipients_controller() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &GRANT)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let dealer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(victim)));
    resolve_top(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    act(&mut game, Action::ChooseOptional { accept: false });
    let source = game.damage_source(dealer);
    game.apply_damage_batch(
        &[DamageEvent {
            source,
            target: Target::Card(victim),
            amount: 1,
            combat: false,
            preventable: true,
        }; 2],
    );
    assert_eq!(game.state.pending_triggers.len(), 1);
    assert_eq!(game.state.pending_triggers[0].controller, opponent);
    let trigger = &game.state.pending_triggers[0];
    assert_eq!(trigger.damage_received.as_ref().unwrap().amount, 2);
}

const PAIN: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: 0,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Any,
                predicate: &TargetPredicate::OtherThanAuraAttachment,
            }),
            program: &[Instruction::AttachedCreaturePowerDamage],
        },
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureDamaged,
            targets: TargetPlan::NONE,
            program: &[Instruction::EventObjectDamageReceivedPlayers(
                PlayerSet::Opponent,
            )],
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn aura_excludes_its_own_creature_and_uses_departure_power_after_a_lethal_response() {
    let pain = Card::from_name("Pacifism").unwrap();
    let _rules = fixture_rules::activate(&[(pain, &PAIN)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut game, player, "Pacifism", Zone::Hand);
    let bolt = put(&mut game, opponent, "Lightning Strike", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(bear)));
    resolve_top(&mut game);
    order_trigger(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    if game.actor() == Some(player) {
        act(&mut game, Action::Pass);
    }
    cast(&mut game, bolt, Some(Target::Card(bear)));
    resolve_top(&mut game);
    order_trigger(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 15);
}

const WARD: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterRelatedUnlessDiscardRandom],
    }],
    ..CardRules::EMPTY
};

#[test]
fn random_ward_has_no_card_selection_and_replays_through_raw_and_index_actions() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &WARD)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let bolt = put(&mut game, opponent, "Lightning Strike", Zone::Hand);
    let hand = [
        put(&mut game, opponent, "Island", Zone::Hand),
        put(&mut game, opponent, "Mountain", Zone::Hand),
    ];
    cast(&mut game, bolt, Some(Target::Card(bear)));
    order_trigger(&mut game);
    resolve_top(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    assert!(
        game.legal_actions()
            .iter()
            .all(|action| { matches!(action, Action::ChooseOptional { .. } | Action::Concede) })
    );
    let snapshot = game.snapshot();
    let mut indexed = game.clone();
    let action = Action::ChooseOptional { accept: true };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(opponent, indexed.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(
        hand.iter()
            .filter(|&&id| game.object_view(id).zone == Zone::Graveyard)
            .count(),
        1
    );
    assert_eq!(game.object_view(bolt).zone, Zone::Stack);
    game.restore(&snapshot).unwrap();
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.object_view(bolt).zone, Zone::Graveyard);
    assert!(
        hand.iter()
            .all(|&id| game.object_view(id).zone == Zone::Hand)
    );
}

const VAULT: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter {
                    kinds: 16,
                    other: true,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlayScaled {
            amount: AmountValue::SacrificeCostManaValueWithBonus(1),
            next_turn: false,
        }],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn paid_artifact_sacrifice_freezes_its_mana_value_for_the_play_permission() {
    let vault_card = Card::from_name("Nutrient Block").unwrap();
    let _rules = fixture_rules::activate(&[(vault_card, &VAULT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let vault = put(&mut game, player, "Nutrient Block", Zone::Battlefield);
    let artifact = put(&mut game, player, "Chrome Companion", Zone::Battlefield);
    let other = put(&mut game, player, "Island", Zone::Library);
    let top = [
        put(&mut game, player, "Island", Zone::Library),
        put(&mut game, player, "Forest", Zone::Library),
        put(&mut game, player, "Mountain", Zone::Library),
    ];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: vault,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let sacrificial = game.object_ref(artifact);
    let source = game.object_ref(vault);
    reject(&mut game, Action::ChooseAdditionalCost { object: source });
    act(
        &mut game,
        Action::ChooseAdditionalCost {
            object: sacrificial,
        },
    );
    pay(&mut game);
    assert!(game.object_view(vault).tapped);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(
        game.state
            .stack
            .last()
            .unwrap()
            .sacrificed_mana_value
            .as_deref(),
        Some(&2)
    );
    game.move_object(vault, Zone::Graveyard);
    resolve_top(&mut game);
    assert_eq!(game.object_view(other).zone, Zone::Library);
    assert!(
        top.iter()
            .all(|&id| game.object_view(id).zone == Zone::Exile)
    );
    act(&mut game, Action::PlayLand { card: top[0] });
}
