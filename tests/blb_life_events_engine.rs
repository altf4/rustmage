//! A single life-change ability observes committed gains and losses.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EVERY_CHANGE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LifeGainedOrLostOwnTurn,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: AmountValue::LifeEventAmount,
        }],
    }],
    ..CardRules::EMPTY
};

const ONE_CHANGE: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::LifeGainedOrLostOwnTurn),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: AmountValue::LifeEventAmount,
        }],
    }],
    ..CardRules::EMPTY
};

const END_STEP: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::GainedOrLostLife,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::GainedAndLostLife,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 2,
            }],
        },
    ],
    ..CardRules::EMPTY
};

const GAIN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 2)],
    ),
    ..CardRules::EMPTY
};
const LOSS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::LoseLife(PlayerSet::Controller, 2)],
    ),
    ..CardRules::EMPTY
};
const DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamagePlayers(PlayerSet::Controller, 2)],
    ),
    ..CardRules::EMPTY
};
const PREVENT_DAMAGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::PreventPlayerDamageUntilNextTurn],
    ),
    ..CardRules::EMPTY
};
const PREVENT_GAIN: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CantGainLife],
    }],
    ..CardRules::EMPTY
};
const PAY_LIFE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            life: 2,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn settle_life_triggers(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    settle(game);
}

fn resolve_spell(game: &mut Game, controller: Player, name: &str) {
    if game.actor() != Some(controller) {
        act(game, Action::Pass);
    }
    assert_eq!(game.actor(), Some(controller));
    let spell = put(game, controller, name, Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    settle_life_triggers(game);
}

fn end_step(game: &mut Game) {
    game.state.step = Step::Main2;
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.step, Step::End);
    settle_life_triggers(game);
}

fn counters(game: &Game, object: CardId) -> u64 {
    game.object_view(object).counters.get(CounterKind::PlusOne)
}

#[test]
fn own_turn_gain_and_loss_share_the_same_once_per_turn_limit() {
    let _rules = fixture_rules::activate(&[
        (named("Bear Cub"), &EVERY_CHANGE),
        (Card::HillGiant, &ONE_CHANGE),
        (named("Opt"), &GAIN),
        (named("Shock"), &LOSS),
    ]);
    for active in [Player::P0, Player::P1] {
        for spells in [["Opt", "Shock"], ["Shock", "Opt"]] {
            let other = active.other();
            let mut game = scenario(active);
            let every = put(&mut game, active, "Bear Cub", Zone::Battlefield);
            let once = put(&mut game, active, "Hill Giant", Zone::Battlefield);
            let off_turn = put(&mut game, other, "Bear Cub", Zone::Battlefield);
            for spell in spells {
                resolve_spell(&mut game, other, spell);
            }
            assert_eq!(counters(&game, off_turn), 0);
            assert_eq!(counters(&game, every), 0);
            assert_eq!(counters(&game, once), 0);

            resolve_spell(&mut game, active, spells[0]);
            assert_eq!(counters(&game, every), 2);
            assert_eq!(counters(&game, once), 2);
            resolve_spell(&mut game, active, spells[1]);
            assert_eq!(counters(&game, every), 4);
            assert_eq!(counters(&game, once), 2);
            assert_eq!(game.state.players[active.index()].life, 20);

            game.record_control(game.object_ref(once), other, None);
            let mountain = put(&mut game, other, "Mountain", Zone::Battlefield);
            end_step(&mut game);
            act(&mut game, Action::Pass);
            act(&mut game, Action::Pass);
            assert_eq!(game.state.active, other);
            assert_eq!(game.state.players[other.index()].life_lost_this_turn, 0);
            act(&mut game, Action::TapMana { card: mountain });
            resolve_spell(&mut game, other, "Shock");
            assert_eq!(counters(&game, off_turn), 2);
            assert_eq!(counters(&game, once), 4);
            assert_eq!(counters(&game, every), 4);
        }
    }
}

#[test]
fn end_step_guard_uses_gross_life_history_and_excludes_prevented_changes() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &END_STEP),
        (named("Bear Cub"), &EVERY_CHANGE),
        (named("Opt"), &GAIN),
        (named("Shock"), &LOSS),
        (named("Lightning Strike"), &DAMAGE),
        (named("Disenchant"), &PREVENT_DAMAGE),
        (named("Swiftfoot Boots"), &PREVENT_GAIN),
    ]);
    for player in [Player::P0, Player::P1] {
        for (spells, expected) in [
            (&[][..], 0),
            (&["Opt"][..], 1),
            (&["Shock"][..], 1),
            (&["Opt", "Shock"][..], 3),
        ] {
            let mut game = scenario(player);
            for &spell in spells {
                resolve_spell(&mut game, player, spell);
            }
            // History belongs to the player and predates the observer's entry.
            let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
            end_step(&mut game);
            assert_eq!(counters(&game, observer), expected, "{player:?} {spells:?}");
        }

        let mut game = scenario(player);
        let every = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        put(
            &mut game,
            player.other(),
            "Swiftfoot Boots",
            Zone::Battlefield,
        );
        resolve_spell(&mut game, player, "Disenchant");
        resolve_spell(&mut game, player, "Opt");
        resolve_spell(&mut game, player, "Lightning Strike");
        resolve_spell(&mut game, player.other(), "Shock");
        let state = &game.state.players[player.index()];
        assert_eq!(state.life, 20);
        assert_eq!(state.life_gained_this_turn, 0);
        assert_eq!(state.life_lost_this_turn, 0);
        assert_eq!(counters(&game, every), 0);
        end_step(&mut game);
        assert_eq!(counters(&game, observer), 0);
    }
}

#[test]
fn paid_life_queues_one_owned_trigger_across_raw_index_and_restore() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &ONE_CHANGE),
        (named("Nutrient Block"), &PAY_LIFE),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let observer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let source = put(&mut game, player, "Nutrient Block", Zone::Battlefield);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(&mut game, Action::PayCosts { payment: [0; 6] });
        let (index, pay) = game
            .legal_actions()
            .into_iter()
            .enumerate()
            .find(|(_, action)| matches!(action, Action::PayCosts { .. }))
            .expect("funded mana and life payment");
        let revision = game.revision();
        let saved = game.snapshot();
        let mut raw = game.clone();
        let mut indexed = game.clone();
        drop(game);
        act(&mut raw, pay.clone());
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[player.index()].life, 18);
        assert_eq!(
            raw.state.stack.last().unwrap().life_event,
            Some(LifeTriggerView { player, amount: 2 })
        );
        settle(&mut raw);
        assert_eq!(raw.state.players[player.index()].life, 20);
        assert_eq!(counters(&raw, observer), 2);

        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, pay);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
