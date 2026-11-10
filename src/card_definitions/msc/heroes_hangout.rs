pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Date Night",
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopChoosePlay {
                amount: 2,
                next_turn: true,
            }],
        },
        SpellMode {
            text: "Patrol Night",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 1,
                max: 2,
            }),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 1,
                    toughness: 0,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::FIRST_STRIKE,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
