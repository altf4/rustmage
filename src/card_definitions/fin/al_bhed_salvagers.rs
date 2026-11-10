pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfOrControlledPermanentDies(1 | 16),
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::LoseLife(PlayerSet::Targeted, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
