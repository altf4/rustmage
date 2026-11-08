// Complete typed source; its independent upkeep delay survives source departure.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 3),
                Instruction::ScheduleStep {
                    step: TriggerStep::Upkeep,
                    relation: CastRelation::You,
                    ability: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::DiscardRandom(PlayerSet::Controller, 3)],
        },
    ],
    ..CardRules::EMPTY
};
