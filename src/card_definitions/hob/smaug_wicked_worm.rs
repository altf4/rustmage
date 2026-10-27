// Complete Oracle and pinned XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensScaled {
                token: Card(5302),
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 1,
                    kind: 16,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
                tapped: true,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::TreasureManaSpent(&TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Any,
                ordinal: None,
            }),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
