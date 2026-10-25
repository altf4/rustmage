// Complete direct typed Oracle behavior; the source remains a permanent recipient.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
            condition: StatValue::Sum(&[
                StatValue::CountAllKinds(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 17,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
                StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Plan"),
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
            ]),
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::MshDoombotToken,
                amount: 2,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
