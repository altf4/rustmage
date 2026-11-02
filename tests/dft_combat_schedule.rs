//! Main-relative extra phases and repeated source-independent combat untaps.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const PROGRAM: &[Instruction] = &[
    Instruction::AdditionalCombatsAfterCurrentMain(2),
    Instruction::ScheduleBeginCombatThisTurn { ability: 0 },
];
const COMBAT_SPELL: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, PROGRAM),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapCreaturesThatAttackedThisTurn],
    }],
    ..CardRules::EMPTY
};
const LEGACY_EXTRA: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::AdditionalCombat]),
    ..CardRules::EMPTY
};
const ABILITY_COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const UNTAP_PROHIBITOR: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttachedUntap(UntapRestriction::All)],
    }],
    ..CardRules::EMPTY
};
fn cast_schedule(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = put(g, p, name, Zone::Hand);
    cast(g, card, None);
    settle(g);
    card
}
fn advance(g: &mut Game) {
    match g.state.pending {
        Pending::Priority => act(g, Action::Pass),
        Pending::Attackers => act(g, Action::FinishAttackers),
        Pending::Blockers => act(g, Action::FinishBlockers),
        Pending::Damage => {
            let range = g.damage_assignment_range().unwrap();
            act(
                g,
                Action::AssignDamage {
                    target: range.target,
                    amount: range.maximum,
                },
            );
        }
        Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
        _ => panic!("unexpected combat scenario choice {:?}", g.state.pending),
    }
}
fn until(g: &mut Game, step: Step) {
    for _ in 0..96 {
        if g.state.step == step
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        advance(g);
    }
    panic!("combat step {step:?} not reached");
}
fn sequence_to_end(g: &mut Game) -> Vec<Step> {
    let mut result = Vec::new();
    let mut previous = g.state.step;
    for _ in 0..160 {
        if g.state.step != previous {
            previous = g.state.step;
            if matches!(
                previous,
                Step::BeginCombat | Step::Main1 | Step::Main2 | Step::End
            ) {
                result.push(previous);
            }
        }
        if g.state.step == Step::End
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
        {
            return result;
        }
        advance(g);
    }
    panic!("extra combat sequence did not terminate");
}
fn attack_to_main2(g: &mut Game, attackers: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &card in attackers {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
    until(g, Step::Main2);
}

#[test]
fn inserted_combats_resume_main1_normal_combat_or_main2_end_without_interleaved_mains() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &COMBAT_SPELL)]);
    for p in [Player::P0, Player::P1] {
        for (main, count) in [(Step::Main1, 3), (Step::Main2, 2)] {
            let mut g = scenario(p);
            g.state.step = main;
            cast_schedule(&mut g, p, "Divination");
            let observed = g.observe(p);
            assert_eq!(observed.additional_combats, 2);
            assert_eq!(
                observed.after_main_combats.unwrap().resume,
                if main == Step::Main1 {
                    Step::BeginCombat
                } else {
                    Step::End
                }
            );
            assert_eq!(observed.delayed_begin_combat.unwrap().len(), 1);
            let steps = sequence_to_end(&mut g);
            let mut expected = vec![Step::BeginCombat; count];
            if main == Step::Main1 {
                expected.push(Step::Main2);
            }
            expected.push(Step::End);
            assert_eq!(steps, expected);
        }
    }
}

#[test]
fn repeated_effects_and_legacy_extra_combat_keep_the_frozen_resume_step() {
    let _rules = fixture_rules::activate(&[
        (Card::Divination, &COMBAT_SPELL),
        (Card::Shock, &LEGACY_EXTRA),
    ]);
    for (main, copies, legacy, count) in [
        (Step::Main1, 2, false, 5),
        (Step::Main2, 2, false, 4),
        (Step::Main2, 1, true, 3),
    ] {
        let mut g = scenario(Player::P0);
        g.state.step = main;
        if legacy {
            cast_schedule(&mut g, Player::P0, "Shock");
        }
        for _ in 0..copies {
            cast_schedule(&mut g, Player::P0, "Divination");
        }
        let steps = sequence_to_end(&mut g);
        let mut expected = vec![Step::BeginCombat; count];
        if main == Step::Main1 {
            expected.push(Step::Main2);
        }
        expected.push(Step::End);
        assert_eq!(
            steps, expected,
            "{main:?}, copies={copies}, legacy={legacy}"
        );
    }
}

#[test]
fn resolving_outside_a_main_creates_only_the_repeated_delay_with_original_controller() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &COMBAT_SPELL)]);
    let mut g = scenario(Player::P0);
    g.state.step = Step::BeginCombat;
    act(&mut g, Action::Pass);
    let source = cast_schedule(&mut g, Player::P1, "Shock");
    g.move_object(source, Zone::Exile);
    let view = g.observe(Player::P0);
    assert_eq!(view.additional_combats, 0);
    assert!(view.after_main_combats.is_none());
    assert_eq!(view.delayed_begin_combat.unwrap()[0].controller, Player::P1);
    g.begin_step(Step::BeginCombat);
    g.prepare_triggers();
    assert_eq!(g.state.stack.last().unwrap().controller, Player::P1);
    assert!(g.state.stack.last().unwrap().targets.is_empty());
    settle(&mut g);
    assert_eq!(g.observe(Player::P1).delayed_begin_combat.unwrap().len(), 1);
}

#[test]
fn delayed_untap_uses_current_exact_attacked_creatures_any_controller_and_shared_restrictions() {
    let _rules = fixture_rules::activate(&[
        (Card::Divination, &COMBAT_SPELL),
        (Card::from_name("Stop Cold").unwrap(), &UNTAP_PROHIBITOR),
    ]);
    let mut g = scenario(Player::P0);
    let a = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let changed = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let blinked = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let noncreature = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let prohibited = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let stunned = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    attack_to_main2(
        &mut g,
        &[a, changed, blinked, noncreature, prohibited, stunned],
    );
    g.state.players[0].mana = [8; 6];
    let source = cast_schedule(&mut g, Player::P0, "Divination");
    g.move_object(source, Zone::Exile);
    g.record_control(g.object_ref(changed), Player::P1, None);
    g.move_object(blinked, Zone::Exile);
    g.move_object(blinked, Zone::Battlefield);
    g.state.objects[blinked as usize].tapped = true;
    let mut recipe = CopyRecipe::new(Card::GoblinPiker);
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(16),
        ..CopyOverrides::NONE
    });
    g.apply_temporary_copy(g.object_ref(noncreature), recipe, CopyDeadline::EndOfTurn);
    let aura = put(&mut g, Player::P0, "Stop Cold", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(prohibited)));
    settle(&mut g);
    g.add_counter(prohibited, CounterKind::Stun, 1);
    g.add_counter(stunned, CounterKind::Stun, 1);
    let later = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    g.state.objects[later as usize].tapped = true;
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::BeginCombat);
    assert!(
        g.object_view(a).tapped,
        "untapping is a counterable delayed trigger"
    );
    settle(&mut g);
    assert!(!g.object_view(a).tapped);
    assert!(!g.object_view(changed).tapped);
    assert!(g.object_view(blinked).tapped);
    assert!(g.object_view(noncreature).tapped);
    assert!(g.object_view(prohibited).tapped);
    assert_eq!(g.counter_count(prohibited, CounterKind::Stun), 1);
    assert!(g.object_view(stunned).tapped);
    assert_eq!(g.counter_count(stunned, CounterKind::Stun), 0);
    assert!(g.object_view(later).tapped);
    g.state.objects[a as usize].tapped = true;
    // Finish the first inserted combat with no declarations; the delay repeats.
    advance(&mut g);
    advance(&mut g); // BeginCombat priority -> attackers.
    assert_eq!(g.state.pending, Pending::Attackers);
    act(&mut g, Action::FinishAttackers);
    until(&mut g, Step::EndCombat);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::BeginCombat);
    settle(&mut g);
    assert!(!g.object_view(a).tapped);
    assert!(!g.object_view(stunned).tapped);
    assert!(g.object_view(later).tapped);
}

#[test]
fn countering_one_delayed_combat_trigger_leaves_future_trigger_active() {
    let _rules = fixture_rules::activate(&[
        (Card::Divination, &COMBAT_SPELL),
        (Card::Counterspell, &ABILITY_COUNTER),
    ]);
    let mut g = scenario(Player::P0);
    let attacker = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    attack_to_main2(&mut g, &[attacker]);
    g.state.players[0].mana = [8; 6];
    cast_schedule(&mut g, Player::P0, "Divination");
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    let trigger = g.state.stack.last().unwrap().id;
    act(&mut g, Action::Pass);
    g.state.players[1].mana = [8; 6];
    let counter = put(&mut g, Player::P1, "Counterspell", Zone::Hand);
    cast(&mut g, counter, Some(Target::Stack(trigger)));
    settle(&mut g);
    assert!(g.object_view(attacker).tapped);
    advance(&mut g);
    advance(&mut g);
    act(&mut g, Action::FinishAttackers);
    until(&mut g, Step::EndCombat);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    settle(&mut g);
    assert!(!g.object_view(attacker).tapped);
    assert_eq!(g.observe(Player::P0).delayed_begin_combat.unwrap().len(), 1);
}

#[test]
fn inserted_phase_and_delayed_packet_schedule_survives_owned_raw_indexed_restore() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &COMBAT_SPELL)]);
    let mut g = scenario(Player::P1);
    g.state.step = Step::Main2;
    let source = cast_schedule(&mut g, Player::P1, "Divination");
    g.move_object(source, Zone::Exile);
    let snapshot = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let mut restored = g.clone();
    restored.restore(&snapshot).unwrap();
    drop(g);
    for (which, branch) in [&mut raw, &mut indexed, &mut restored]
        .into_iter()
        .enumerate()
    {
        let index = branch
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        if which == 1 {
            branch
                .apply_index(branch.actor().unwrap(), branch.revision(), index)
                .unwrap();
        } else {
            act(branch, Action::Pass);
        }
        assert_eq!(
            sequence_to_end(branch),
            vec![Step::BeginCombat, Step::BeginCombat, Step::End]
        );
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.snapshot(), restored.snapshot());
}

#[test]
fn cleanup_expires_both_extra_phase_and_repeated_delayed_obligations() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &COMBAT_SPELL)]);
    let mut g = scenario(Player::P0);
    cast_schedule(&mut g, Player::P0, "Divination");
    assert!(g.observe(Player::P1).delayed_begin_combat.is_some());
    g.finish_cleanup();
    let view = g.observe(Player::P1);
    assert_eq!(view.additional_combats, 0);
    assert!(view.after_main_combats.is_none());
    assert!(view.delayed_begin_combat.is_none());
    g.begin_step(Step::BeginCombat);
    g.prepare_triggers();
    assert!(g.state.stack.is_empty());
    assert!(g.state.pending_triggers.is_empty());
}
