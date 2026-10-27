pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 3,
                toughness: 0,
            },
            Instruction::ManifestDreadWhenTargetDies,
        ],
    ),
    ..CardRules::EMPTY
};
