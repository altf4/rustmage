// Complete printed behavior; mode combinations resolve in printed order.
// Base mana, identity and format metadata come from the pinned card facts.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Counter and evasion",
            targets: one(TargetRule::Creature),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::CANT_BE_BLOCKED,
                },
            ],
        },
        SpellMode {
            text: "Owner chooses library end",
            targets: one(object(17)),
            program: &[Instruction::OwnerLibrary],
        },
        SpellMode {
            text: "Counter and evasion + Owner chooses library end",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Creature),
                TargetSpec::one(object(17)),
            ]),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::CANT_BE_BLOCKED,
                },
                Instruction::SelectGroup(1),
                Instruction::OwnerLibrary,
            ],
        },
    ]),
    mode_costs: Some(&[generic(3), generic(2), generic(5)]),
    ..CardRules::EMPTY
};
