// Complete printed behavior from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 5),
    keywords: Keywords::NONE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Recruit { token: Card(5241) }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(
            0,
            DynamicStats {
                power: Some(StatExpr {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 1,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    multiplier: 1,
                    offset: 0,
                }),
                toughness: None,
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Cda,
            fragments: &[StaticAbilityFragment::DynamicStats(
                0,
                DynamicStats {
                    power: Some(StatExpr {
                        value: StatValue::Count(CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 1,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: false,
                        }),
                        multiplier: 1,
                        offset: 0,
                    }),
                    toughness: None,
                },
            )],
        }],
        dynamic_stats: Some(DynamicStats {
            power: Some(StatExpr {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
                multiplier: 1,
                offset: 0,
            }),
            toughness: None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
