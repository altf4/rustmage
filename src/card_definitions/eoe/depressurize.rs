pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: -3,
                toughness: 0,
            },
            Instruction::DestroyTargetsAtMostPower(0),
        ],
    ),
    ..CardRules::EMPTY
};
