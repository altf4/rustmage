// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
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
            condition: TriggerCondition::SelfBecomesTapped,
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[Instruction::Mill(
                PlayerSet::Targeted,
                MillAmount::Amount(AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Lesson"),
                    basic: false,
                    other: false,
                    cards_only: true,
                }))),
            )],
        },
    ],
    ..CardRules::EMPTY
};
