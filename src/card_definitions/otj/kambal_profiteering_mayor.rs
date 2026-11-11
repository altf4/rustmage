pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::TokensEnterOpponent),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTappedCopiesOfEntryBatch],
        },
        TriggeredAbility {
            condition: TriggerCondition::TokensEnterYou,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLife(PlayerSet::Opponent, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
