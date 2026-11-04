pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLife(PlayerSet::Opponent, 2),
                Instruction::GainLife(PlayerSet::Controller, 2),
                Instruction::Surveil(2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SubtypeEntersYou("Dragon"),
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnSourceToHand(TargetZone::Battlefield)],
        },
    ],
    ..CardRules::EMPTY
};
