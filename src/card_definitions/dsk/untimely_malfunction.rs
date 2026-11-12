pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target artifact",
            targets: one(object(16)),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Change the target of a spell or ability",
            targets: one(TargetRule::SingleTargetStackObject),
            program: &[Instruction::ChangeSingleStackTarget],
        },
        SpellMode {
            text: "One or two creatures can't block",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 1,
                max: 2,
            }),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BLOCK,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
