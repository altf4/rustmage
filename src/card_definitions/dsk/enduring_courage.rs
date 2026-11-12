pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OtherCreatureEntersYou,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Grow {
                    subject: Subject::Entering,
                    power: 2,
                    toughness: 0,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Entering,
                    keywords: Keywords::HASTE,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfCreatureDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnSourceAsEnchantment],
        },
    ],
    ..CardRules::EMPTY
};
