pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ControlledSubtypeAttackDeclaration("Bird"),
            targets: TargetPlan::NONE,
            program: &[Instruction::LookTopOneToHandThenLandsTapped(
                AmountValue::DeclaredAttackerCount,
            )],
        },
        TriggeredAbility {
            condition: TriggerCondition::LandEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 0,
            }],
        },
    ],
    ..CardRules::EMPTY
};
