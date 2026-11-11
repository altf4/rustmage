// Direct typed behavior from the paired Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CastFromHand(&TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Instant,
                ordinal: None,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantCastRebound],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnDepartedSourceTransformedTapped {
                counters: &[(CounterKind::Time, 3)],
            }],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
