//! Shared counter parity and one-shot signed group base-stat sampling.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const EVEN: StepGuard = StepGuard::Expression {
    value: StatValue::Parity {
        value: &StatValue::SourceTotalCounters,
        odd: false,
    },
    minimum: Some(1),
    maximum: None,
};
const PARITY_DRAW: &[Instruction] = &[
    Instruction::PutCounters {
        subject: Subject::Source,
        kind: CounterKind::PlusOne,
        amount: 1,
    },
    Instruction::IfAmountParity {
        value: AmountValue::SourceTotalCounters,
        odd: true,
        skip: 1,
    },
    Instruction::DrawController(2),
];
const COUNTER_PARITY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(
            0,
            CombatRestriction {
                subject: Subject::Source,
                attack: true,
                block: true,
                unless: Some(EVEN),
            },
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: PARITY_DRAW,
    }],
    ..CardRules::EMPTY
};
const OWN_CREATURES: &[BattlefieldFilter] = &[BattlefieldFilter {
    kinds: 1,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
}];
const SET_GROUP: Instruction = Instruction::SetPermanentGroupBaseStats {
    filters: OWN_CREATURES,
    exclude_source: true,
    power: Some(ResolvedStatValue::SourcePower),
    toughness: Some(ResolvedStatValue::SourcePower),
};
const SET_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: true,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        colored: [0, 1, 0, 0, 2, 0],
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::ActivationX,
        Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: AmountValue::ChosenX,
        },
        Instruction::May { skip: 1 },
        SET_GROUP,
    ],
    sorcery: false,
    once_per_turn: false,
};
const SETTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[SET_ABILITY],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const NEGATIVE_SETTER: CardRules = CardRules {
    activated: &[ActivatedAbility {
        program: &[Instruction::May { skip: 1 }, SET_GROUP],
        ..SET_ABILITY
    }],
    ..SETTER
};
const LAND_SETTER: CardRules = CardRules {
    effect: Effect::Land,
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        program: &[SET_GROUP],
        ..SET_ABILITY
    }],
    ..CardRules::EMPTY
};

fn activate(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if g.legal_actions().contains(&Action::ChooseX { value: 2 }) {
        act(g, Action::ChooseX { value: 2 });
    }
    pay(g);
}
fn optional(g: &mut Game) {
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::EffectOptional { .. }) {
            return;
        }
        if let Some(action) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
        {
            act(g, action);
        } else {
            assert_eq!(g.state.pending, Pending::Priority);
            act(g, Action::Pass);
        }
    }
    panic!("optional effect did not arrive");
}
fn finish(g: &mut Game) {
    for _ in 0..48 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let action = g.legal_actions().into_iter().find(|a| {
            matches!(
                a,
                Action::ChooseReplacement { .. } | Action::PutTrigger { .. }
            )
        });
        if let Some(action) = action {
            act(g, action);
        } else {
            assert_eq!(g.state.pending, Pending::Priority);
            act(g, Action::Pass);
        }
    }
    panic!("bounded shared-rule resolution failed");
}
fn main_trigger(g: &mut Game) {
    g.state.step = Step::Draw;
    g.begin_step(Step::Main1);
    g.prepare_triggers();
}

#[test]
fn total_counter_parity_includes_zero_hot_and_named_kinds_for_attack_and_block() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &COUNTER_PARITY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        assert!(g.can_declare_attacker(p, source), "zero is even");
        for (kind, even) in [
            (CounterKind::Charge, false),
            (CounterKind::Stun, true),
            (CounterKind::Lore, false),
            (CounterKind::Bait, true),
        ] {
            g.add_counter(source, kind, 1);
            assert_eq!(g.can_declare_attacker(p, source), even, "{kind:?}");
            g.state.active = p.other();
            assert_eq!(g.can_block_pair(source, enemy), even, "{kind:?}");
            g.state.active = p;
        }
        let total = g.counter_amounts(source).total();
        assert_eq!(total, 4);
        assert_eq!(
            g.stat_value(StatValue::SourceTotalCounters, g.object_ref(source), p),
            4
        );
    }
}

#[test]
fn parity_restriction_is_removed_with_its_printed_ability() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &COUNTER_PARITY)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.add_counter(source, CounterKind::Charge, 1);
    assert!(!g.can_declare_attacker(Player::P0, source));
    let aura = put(&mut g, Player::P0, "Stop Cold", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(source)));
    finish(&mut g);
    g.untap_objects(&[source], untap_rules::UntapCause::Effect);
    assert!(g.can_declare_attacker(Player::P0, source));
    g.move_object(aura, Zone::Graveyard);
    assert!(!g.can_declare_attacker(Player::P0, source));
}

#[test]
fn post_placement_parity_reads_replaced_counter_result_before_drawing() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &COUNTER_PARITY)]);
    for doubled in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        if doubled {
            put(&mut g, Player::P0, "Doubling Season", Zone::Battlefield);
        }
        main_trigger(&mut g);
        finish(&mut g);
        assert_eq!(
            g.counter_count(source, CounterKind::PlusOne),
            if doubled { 2 } else { 1 }
        );
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            if doubled { 0 } else { 2 }
        );
    }
}

#[test]
fn total_counter_departure_lki_is_latest_exact_and_owned_across_new_incarnation() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &COUNTER_PARITY)]);
    for last_total in [4, 5] {
        let mut g = scenario(Player::P1);
        let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        g.add_counter(source, CounterKind::Charge, 2);
        main_trigger(&mut g);
        g.add_counter(source, CounterKind::Stun, last_total - 2);
        let old = g.object_ref(source);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        g.add_counter(
            source,
            CounterKind::Bait,
            if last_total % 2 == 0 { 1 } else { 2 },
        );
        assert_ne!(old, g.object_ref(source));
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
            finish(branch);
            assert_eq!(
                branch.state.players[1].cards_drawn_this_turn,
                if last_total % 2 != 0 { 2 } else { 0 }
            );
            assert_eq!(
                branch.counter_count(source, CounterKind::PlusOne),
                0,
                "new incarnation is not the counter recipient"
            );
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.snapshot(), restored.snapshot());
    }
}

#[test]
fn total_counter_quantity_opts_in_through_conditional_headers_and_nested_modal_programs() {
    let program = &[Instruction::Modal(&[SpellMode {
        text: "parity",
        targets: TargetPlan::NONE,
        program: &[Instruction::IfAmountParity {
            value: AmountValue::Sum(&[AmountValue::SourceTotalCounters]),
            odd: true,
            skip: 0,
        }],
    }])];
    let facts = card_quantities::counter_quantity_facts(program, |kind| {
        if kind == CounterKind::Charge { 7 } else { 0 }
    })
    .unwrap();
    assert_eq!(facts.len(), CounterKind::ALL.len());
    assert_eq!(facts.iter().map(|r| u128::from(r.count)).sum::<u128>(), 7);
}

#[test]
fn group_base_stats_samples_source_power_after_counters_and_replacements_once() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SETTER)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let opponent = put(&mut g, Player::P1, "Goblin Piker", Zone::Battlefield);
    g.add_counter(recipient, CounterKind::PlusOne, 1);
    put(&mut g, Player::P0, "Doubling Season", Zone::Battlefield);
    activate(&mut g, source);
    optional(&mut g);
    assert_eq!(stats(&g, source), (6, 6));
    act(&mut g, Action::ChooseOptional { accept: true });
    finish(&mut g);
    assert_eq!(
        stats(&g, recipient),
        (7, 7),
        "set base, then apply its own counter"
    );
    assert_eq!(stats(&g, opponent), (2, 1));
    let later = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    g.state.objects[source as usize].growth.power = 10;
    assert_eq!(stats(&g, recipient), (7, 7));
    assert_eq!(stats(&g, later), (2, 1));
    g.move_object(recipient, Zone::Exile);
    g.move_object(recipient, Zone::Battlefield);
    assert_eq!(
        stats(&g, recipient),
        (2, 1),
        "frozen recipient does not follow a new object"
    );
}

#[test]
fn optional_group_decline_keeps_paid_counter_effect_and_original_other_stats() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SETTER)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    activate(&mut g, source);
    optional(&mut g);
    act(&mut g, Action::ChooseOptional { accept: false });
    finish(&mut g);
    assert_eq!(stats(&g, source), (4, 4));
    assert_eq!(stats(&g, recipient), (2, 1));
}

#[test]
fn group_source_power_is_signed_and_departure_lki_survives_owned_optional_choice() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &NEGATIVE_SETTER)]);
    let mut g = scenario(Player::P1);
    let source = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let recipient = put(&mut g, Player::P1, "Goblin Piker", Zone::Battlefield);
    g.state.objects[source as usize].growth.power = -4;
    g.add_counter(recipient, CounterKind::PlusOne, 3);
    activate(&mut g, source);
    optional(&mut g);
    g.move_object(source, Zone::Exile);
    g.move_object(source, Zone::Battlefield);
    g.state.objects[source as usize].growth.power = 20;
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
            .position(|a| *a == Action::ChooseOptional { accept: true })
            .unwrap();
        if which == 1 {
            branch
                .apply_index(branch.actor().unwrap(), branch.revision(), index)
                .unwrap();
        } else {
            act(branch, Action::ChooseOptional { accept: true });
        }
        finish(branch);
        assert_eq!(
            stats(branch, recipient),
            (1, 1),
            "signed -2 base plus three counters"
        );
        assert_eq!(stats(branch, source), (22, 2));
        branch.finish_cleanup();
        assert_eq!(stats(branch, recipient), (5, 4));
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.snapshot(), restored.snapshot());
}

#[test]
fn current_exact_noncreature_source_samples_zero_power() {
    let _rules = fixture_rules::activate(&[(Card::Forest, &LAND_SETTER)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let recipient = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.add_counter(recipient, CounterKind::PlusOne, 1);
    activate(&mut g, source);
    finish(&mut g);
    assert_eq!(stats(&g, recipient), (1, 1));
}
