pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersIfCast,
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileEachLibraryExceptBottomFaceDown(6)],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
