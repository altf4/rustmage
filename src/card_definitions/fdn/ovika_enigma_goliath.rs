// Complete Oracle body authored from the paired pinned XMage source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayManaLife {
                cost: generic(3),
                life: 3,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensWithTemporaryKeywords {
                token: Card(5343),
                amount: AmountValue::CastSpellManaValue,
                keywords: Keywords::HASTE,
            }],
        },
    ],
    ..CardRules::EMPTY
};
