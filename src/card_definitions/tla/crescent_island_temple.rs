// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensScaled {
                token: Card(5709),
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Shrine"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OtherSubtypeEntersYou("Shrine"),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5709),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
