// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CaptureEntryResult,
            Instruction::PutHandFiltered {
                filter: SearchFilter::Kinds(1 | 16),
                skip: 2,
            },
            Instruction::GrantEntryResultKeywords {
                keywords: Keywords::HASTE,
                permanent: true,
            },
            Instruction::GrantEntryResultTriggers(&[0, 1]),
        ],
    ),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::SelfLeaves),
            targets: TargetPlan::NONE,
            program: &[Instruction::DamageEachCreatureByDepartureManaValue],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSource],
        },
    ],
    ..CardRules::EMPTY
};
