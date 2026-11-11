pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::SecondMainPhase,
            event: &TriggerCondition::AtStep {
                step: TriggerStep::Main2,
                relation: CastRelation::You,
                guard: StepGuard::OpponentLostLife,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopChoosePlay {
            amount: 2,
            next_turn: false,
        }],
    }],
    ..CardRules::EMPTY
};
