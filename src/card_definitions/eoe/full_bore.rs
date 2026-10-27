// Directly authored from the complete Oracle and XMage source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 3,
                toughness: 2,
            },
            Instruction::GrantWarpedTargetsKeywords(Keywords::TRAMPLE.union(Keywords::HASTE)),
        ],
    ),
    ..CardRules::EMPTY
};
