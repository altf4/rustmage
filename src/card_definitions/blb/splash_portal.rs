pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        &[
            Instruction::CaptureEntryResult,
            Instruction::BlinkFollowingPublicMove { all_groups: false },
            Instruction::IfEntryResultSubtype {
                subtypes: &["Bird", "Frog", "Otter", "Rat"],
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
