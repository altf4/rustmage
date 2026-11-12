pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 2 },
                Instruction::Grow {
                    subject: Subject::Source,
                    power: 4,
                    toughness: 0,
                },
                Instruction::ScheduleStep {
                    step: TriggerStep::End,
                    relation: CastRelation::Any,
                    ability: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSource],
        },
    ],
    ..CardRules::EMPTY
};
