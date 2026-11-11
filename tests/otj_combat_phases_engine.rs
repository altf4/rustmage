//! Additional phase order and actual combat-damage callback receipts.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const EXTRA_UPKEEP: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AdditionalUpkeepsAfterCombat(
            AmountValue::DamageEventAmount,
        )],
    }],
    ..CardRules::EMPTY
};
const WATCH_UPKEEP: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const EXTRA_COMBAT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::UntapPlayerCreatures(PlayerSet::Controller),
            Instruction::If {
                guard: StepGuard::YourCombatPhase,
                skip: 1,
            },
            Instruction::AdditionalCombat,
        ],
    ),
    ..CardRules::EMPTY
};
const COPY_ABILITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};
const DAMAGE_CALLBACK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[Instruction::ScheduleTargetPlayerCombatDamageTrigger { ability: 0 }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5302),
            amount: 1,
            tapped: true,
        }],
    }],
    ..CardRules::EMPTY
};

fn reach(g: &mut Game, checkpoint: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if checkpoint(g) {
            return;
        }
        match g.state.pending {
            Pending::Priority => act(g, Action::Pass),
            Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
            ref other => panic!("unexpected checkpoint state {other:?}"),
        }
    }
    panic!("phase scenario exceeded 80 actions");
}

fn attackers(g: &mut Game) {
    reach(g, |g| g.state.pending == Pending::Attackers);
}

fn finish_blocks(g: &mut Game) {
    reach(g, |g| g.state.pending == Pending::Blockers);
    act(g, Action::FinishBlockers);
}

fn at_priority(g: &Game, step: Step) -> bool {
    g.state.step == step && g.state.pending == Pending::Priority && g.state.stack.is_empty()
}

#[test]
fn combat_damage_schedules_exact_upkeeps_after_the_newer_combat_without_draws() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXTRA_UPKEEP),
        (Card::GoblinPiker, &WATCH_UPKEEP),
        (Card::GiantGrowth, &EXTRA_COMBAT),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[bear as usize].gained_keywords = Keywords::DOUBLE_STRIKE;
    let forest = put(&mut g, p, "Forest", Zone::Battlefield);
    put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let spell = put(&mut g, p, "Giant Growth", Zone::Hand);
    let starting_life = g.state.players[p.index()].life;
    let opponent_life = g.state.players[p.other().index()].life;
    let library = g.state.players[p.index()].library.len();

    attackers(&mut g);
    act(&mut g, Action::Attack { card: bear });
    act(&mut g, Action::FinishAttackers);
    finish_blocks(&mut g);
    reach(&mut g, |g| at_priority(g, Step::EndCombat));
    assert_eq!(g.state.players[p.other().index()].life, opponent_life - 4);
    assert_eq!(
        g.observe(p).after_combat_phases.unwrap(),
        [AfterCombatPhaseView {
            first_step: Step::Upkeep,
            remaining: 4
        }]
    );

    act(&mut g, Action::TapMana { card: forest });
    cast(&mut g, spell, None);
    settle(&mut g);
    assert!(!g.object_view(bear).tapped);
    assert_eq!(
        g.observe(p).after_combat_phases.unwrap(),
        [
            AfterCombatPhaseView {
                first_step: Step::BeginCombat,
                remaining: 1
            },
            AfterCombatPhaseView {
                first_step: Step::Upkeep,
                remaining: 4
            },
        ]
    );

    // The later combat happens immediately, before any added upkeep.
    reach(&mut g, |g| at_priority(g, Step::BeginCombat));
    assert_eq!(g.state.players[p.index()].life, starting_life);
    attackers(&mut g);
    act(&mut g, Action::FinishAttackers);
    reach(&mut g, |g| at_priority(g, Step::Upkeep));

    let checkpoint = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let mut restored = g.clone();
    drop(g);
    act(&mut raw, Action::Pass);
    let pass = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    indexed
        .apply_index(indexed.actor().unwrap(), indexed.revision(), pass)
        .unwrap();
    restored.restore(&checkpoint).unwrap();
    act(&mut restored, Action::Pass);
    for game in [&mut raw, &mut indexed, &mut restored] {
        reach(game, |g| at_priority(g, Step::Main2));
        assert_eq!(game.state.players[p.index()].life, starting_life + 4);
        assert_eq!(game.state.players[p.index()].library.len(), library);
        assert_eq!(game.state.turn, 3);
        assert!(game.object_view(forest).tapped);
        assert!(game.observe(p).after_combat_phases.is_none());
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.snapshot(), restored.snapshot());
}

#[test]
fn a_nonactive_ability_copy_cannot_add_upkeeps_to_the_active_turn() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXTRA_UPKEEP),
        (Card::Counterspell, &COPY_ABILITY),
        (Card::GiantGrowth, &EXTRA_COMBAT),
    ]);
    let p = Player::P1;
    let mut g = scenario(p);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let nonactive_spell = put(&mut g, p.other(), "Giant Growth", Zone::Hand);
    let forest = put(&mut g, p.other(), "Forest", Zone::Battlefield);
    let islands = [
        put(&mut g, p.other(), "Island", Zone::Battlefield),
        put(&mut g, p.other(), "Island", Zone::Battlefield),
    ];

    attackers(&mut g);
    act(&mut g, Action::Attack { card: bear });
    act(&mut g, Action::FinishAttackers);
    finish_blocks(&mut g);
    reach(&mut g, |g| {
        g.state.step == Step::CombatDamage && !g.state.stack.is_empty()
    });
    let trigger = g.state.stack.last().unwrap().id;
    act(&mut g, Action::Pass);
    for card in islands {
        act(&mut g, Action::TapMana { card });
    }
    let copy = put(&mut g, p.other(), "Counterspell", Zone::Hand);
    cast(&mut g, copy, Some(Target::Stack(trigger)));
    settle(&mut g);
    assert_eq!(
        g.observe(p).after_combat_phases.unwrap(),
        [AfterCombatPhaseView {
            first_step: Step::Upkeep,
            remaining: 2
        }]
    );

    reach(&mut g, |g| at_priority(g, Step::EndCombat));
    act(&mut g, Action::Pass);
    act(&mut g, Action::TapMana { card: forest });
    cast(&mut g, nonactive_spell, None);
    settle(&mut g);
    assert_eq!(g.observe(p).additional_combats, 0);
}

#[test]
fn a_turn_callback_counts_each_actual_hit_on_the_named_player_and_expires() {
    let _rules = fixture_rules::activate(&[(Card::GiantGrowth, &DAMAGE_CALLBACK)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let first = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[first as usize].gained_keywords = Keywords::DOUBLE_STRIKE;
    let second = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    let zero = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
    g.state.objects[zero as usize].growth.power = -2;
    let walker = put(&mut g, p.other(), "Oko, the Ringleader", Zone::Battlefield);
    g.set_counters(walker, Counters::of(CounterKind::Loyalty, 6));
    let defender = g.object_ref(walker);
    let spell = put(&mut g, p, "Giant Growth", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(p.other())));
    settle(&mut g);

    attackers(&mut g);
    act(&mut g, Action::Attack { card: first });
    act(&mut g, Action::Attack { card: zero });
    act(
        &mut g,
        Action::AttackPlaneswalker {
            card: second,
            defender,
        },
    );
    act(&mut g, Action::FinishAttackers);
    finish_blocks(&mut g);
    reach(&mut g, |g| at_priority(g, Step::Main2));

    let treasure: Vec<_> = g
        .state
        .objects
        .iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == Card(5302))
        .collect();
    assert_eq!(treasure.len(), 2);
    assert!(
        treasure
            .iter()
            .all(|object| object.tapped && object.controller == p)
    );
    assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 4);
    g.finish_cleanup();
    assert!(g.observe(p).turn_callbacks.is_none());
}
