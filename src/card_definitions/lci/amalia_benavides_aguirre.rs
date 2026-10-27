// Direct typed behavior from Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::LifeGained {
                first_only: false,
                own_turn: false,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Explore(Subject::Source),
                Instruction::DestroyOthersIfSourcePowerTwenty,
            ],
        },
    ],
    ..CardRules::EMPTY
};
