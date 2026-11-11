pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 3,
        }),
        &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BLOCK,
            },
            Instruction::Discover(4),
        ],
    ),
    ..CardRules::EMPTY
};
