// Oracle restricts the player target to an opponent; XMage's TargetPlayer is broader.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            }),
            TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::RevealTargetHandChoose {
                kinds: u16::MAX & !2,
                exile: true,
            },
            Instruction::SelectGroup(1),
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
