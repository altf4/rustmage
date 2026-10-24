// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ManaSpent {
                minimum: 0,
                increment: true,
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::Any,
                    ordinal: None,
                },
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
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensWithSourceCounters {
                token: Card::SosFractalToken,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
