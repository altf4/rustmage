// The temporary land form keeps abilities and ends at this controller's next turn.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::SourceCreature,
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::Colors {
                        filter: &CastFilter::Any,
                        mask: 2,
                    },
                    ordinal: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::UntapSource,
                Instruction::BecomeSourceLandUntilNextTurn,
            ],
        },
    ],
    ..CardRules::EMPTY
};
