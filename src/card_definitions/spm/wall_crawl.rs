// Direct typed rules from the pinned Oracle and XMage sources.
const SPIDERS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Spider"),
    basic: false,
    other: false,
    cards_only: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokens {
                token: Card(5544),
                amount: 1,
                tapped: false,
            },
            Instruction::GainLifeScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::Count(SPIDERS)),
            ),
        ],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ControlledCreatureScaling {
                subtype: Some("Spider"),
                value: StatValue::Fixed(1),
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            }],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[
                StaticAbilityFragment::ControlledSubtypeCannotBeBlockedByKeyword {
                    subtype: "Spider",
                    blocker_keyword: Keywords::DEFENDER,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
