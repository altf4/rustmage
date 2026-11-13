pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCombatRed(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::BendsYou,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::IfAmountAtLeast {
                    value: AmountValue::Stat(StatValue::ElementsBentThisTurn),
                    minimum: 4,
                    skip: 1,
                },
                Instruction::TransformSource,
            ],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
