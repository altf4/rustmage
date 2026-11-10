pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessDiscardRandom],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::VoidActive,
                event: &TriggerCondition::SelfAttacksDefendingPlayer,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::LoseHalfLifeRoundedUp(PlayerSet::EventPlayer)],
        },
    ],
    ..CardRules::EMPTY
};
