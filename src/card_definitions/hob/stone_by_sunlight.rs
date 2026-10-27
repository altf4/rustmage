// Complete pinned Oracle and XMage modes; artifact addition does not grant creature.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target creature with power 4 or greater.",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::PowerAtLeast(4),
            }),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Until end of turn, target creature becomes an artifact in addition to its other types and gains indestructible.",
            targets: one(TargetRule::Creature),
            program: &[
                Instruction::AddArtifactToTargets,
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::INDESTRUCTIBLE,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
