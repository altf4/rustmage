pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::GainControlUntilEndTurn,
            Instruction::Untap,
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
            Instruction::Suspect {
                subject: Subject::Targets,
            },
        ],
    ),
    extras: &CardExtras {
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
