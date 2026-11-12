// Direct typed behavior from the paired Oracle and pinned XMage sources.
const BASIC: SearchSpec = SearchSpec {
    filter: SearchFilter::BasicLand,
    maximum: 1,
    destination: SearchDestination::Battlefield,
    tapped: true,
    reveal: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::SearchScaled {
                spec: BASIC,
                maximum: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Shrine"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OtherSubtypeEntersYou("Shrine"),
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(BASIC)],
        },
    ],
    ..CardRules::EMPTY
};
