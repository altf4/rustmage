// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
