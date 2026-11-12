pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Eerie,
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 2,
                toughness: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: one(TargetRule::Any),
            program: &[Instruction::DamageScaled(AmountValue::SourcePower)],
        },
    ],
    ..CardRules::EMPTY
};
