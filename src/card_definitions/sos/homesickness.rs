// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
            TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 2,
            },
        ]),
        &[
            Instruction::SelectGroup(0),
            Instruction::Draw(PlayerSet::Targeted, 2),
            Instruction::SelectGroup(1),
            Instruction::Tap,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Stun,
                amount: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
