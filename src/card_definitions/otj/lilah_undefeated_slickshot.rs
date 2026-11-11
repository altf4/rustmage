pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::CastFromHand(&TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::MulticoloredInstantSorcery,
                ordinal: None,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantCastPlotOnResolution],
        },
    ],
    ..CardRules::EMPTY
};
