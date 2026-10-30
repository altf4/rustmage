// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 2,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FLYING,
            },
            Instruction::EnsureJaceToken,
            Instruction::EmpowerJace(2),
        ],
    ),
    ..CardRules::EMPTY
};
