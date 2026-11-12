const SUSPECTED: TargetRule = TargetRule::Filtered {
    base: &TargetRule::creature_you_control(),
    predicate: &TargetPredicate::Suspected,
};
const COUNTER: Instruction = Instruction::PutCounters {
    subject: Subject::Targets,
    kind: CounterKind::PlusOne,
    amount: 1,
};
const IF_STILL_SUSPECTED: Instruction = Instruction::IfSubjectSuspected {
    subject: Subject::Targets,
    skip: 2,
};
const MAY: Instruction = Instruction::May { skip: 1 };
const UNSUSPECT: Instruction = Instruction::Unsuspect {
    subject: Subject::Targets,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target creature.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Put a +1/+1 counter on target suspected creature you control. You may have it become no longer suspected.",
            targets: TargetPlan::Single(TargetSpec::one(SUSPECTED)),
            program: &[COUNTER, IF_STILL_SUSPECTED, MAY, UNSUSPECT],
        },
        SpellMode {
            text: "Choose both modes.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Creature),
                TargetSpec::one(SUSPECTED),
            ]),
            program: &[
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                COUNTER,
                IF_STILL_SUSPECTED,
                MAY,
                UNSUSPECT,
            ],
        },
    ]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
