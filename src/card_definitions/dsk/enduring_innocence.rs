pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::LIFELINK,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::FilteredEntersBatchYou {
                predicate: EntryPredicate::CreaturePowerAtMost(2),
                other: true,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfCreatureDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnSourceAsEnchantment],
        },
    ],
    ..CardRules::EMPTY
};
