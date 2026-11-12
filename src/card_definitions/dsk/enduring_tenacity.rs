pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::LifeGained {
                first_only: false,
                own_turn: false,
            },
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::LoseGainedLife(PlayerSet::Targeted)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfCreatureDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnSourceAsEnchantment],
        },
    ],
    ..CardRules::EMPTY
};
