// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Targeted {
                    filter: &CastFilter::InstantSorcery,
                    target: CastTarget::Creature(CastRelation::Any),
                },
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDiesWithCounters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::PutSourceCountersOnTargets],
        },
    ],
    ..CardRules::EMPTY
};
