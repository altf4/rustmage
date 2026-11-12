// Direct typed behavior from the pinned Oracle and XMage sources.
const SHRINK: Instruction = Instruction::Grow {
    subject: Subject::Targets,
    power: -1,
    toughness: -1,
};
const COUNTER: Instruction = Instruction::PutCounters {
    subject: Subject::Targets,
    kind: CounterKind::PlusOne,
    amount: 1,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target creature gets -1/-1 until end of turn.",
            targets: one(TargetRule::Creature),
            program: &[SHRINK],
        },
        SpellMode {
            text: "Put a +1/+1 counter on target creature.",
            targets: one(TargetRule::Creature),
            program: &[COUNTER],
        },
        SpellMode {
            text: "Both modes",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Creature),
                TargetSpec::one(TargetRule::Creature),
            ]),
            program: &[SHRINK, Instruction::SelectGroup(1), COUNTER],
        },
    ]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
