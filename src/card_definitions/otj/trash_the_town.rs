const CREATURE: TargetSpec = TargetSpec::one(TargetRule::Creature);
const COUNTERS: Instruction = Instruction::PutCounters {
    subject: Subject::Targets,
    kind: CounterKind::PlusOne,
    amount: 2,
};
const TRAMPLE: Instruction = Instruction::GrantKeywords {
    subject: Subject::Targets,
    keywords: Keywords::TRAMPLE,
};
const DRAW_TRIGGER: Instruction = Instruction::GrantUntilEndTurn { ability: 0 };
const fn mode(
    text: &'static str,
    targets: TargetPlan,
    program: &'static [Instruction],
) -> SpellMode {
    SpellMode {
        text,
        targets,
        program,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        mode(
            "Put two +1/+1 counters on a creature",
            TargetPlan::Single(CREATURE),
            &[COUNTERS],
        ),
        mode("Grant trample", TargetPlan::Single(CREATURE), &[TRAMPLE]),
        mode(
            "Grant the combat damage draw ability",
            TargetPlan::Single(CREATURE),
            &[DRAW_TRIGGER],
        ),
        mode(
            "Put counters on a creature and grant trample",
            TargetPlan::Groups(&[CREATURE, CREATURE]),
            &[COUNTERS, Instruction::SelectGroup(1), TRAMPLE],
        ),
        mode(
            "Put counters on a creature and grant the draw ability",
            TargetPlan::Groups(&[CREATURE, CREATURE]),
            &[COUNTERS, Instruction::SelectGroup(1), DRAW_TRIGGER],
        ),
        mode(
            "Grant trample and the draw ability",
            TargetPlan::Groups(&[CREATURE, CREATURE]),
            &[TRAMPLE, Instruction::SelectGroup(1), DRAW_TRIGGER],
        ),
        mode(
            "Put counters on a creature and grant trample and the draw ability",
            TargetPlan::Groups(&[CREATURE, CREATURE, CREATURE]),
            &[
                COUNTERS,
                Instruction::SelectGroup(1),
                TRAMPLE,
                Instruction::SelectGroup(2),
                DRAW_TRIGGER,
            ],
        ),
    ]),
    mode_counts: Some(&[1, 1, 1, 2, 2, 2, 3]),
    mode_costs: Some(&[
        generic(2),
        generic(1),
        generic(1),
        generic(3),
        generic(3),
        generic(2),
        generic(4),
    ]),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};
