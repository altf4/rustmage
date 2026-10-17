// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::RevealUntilLandToGraveyard(PlayerSet::Opponent)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(
            0,
            DynamicStats {
                power: Some(StatExpr {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 1,
                        kind: 0,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    multiplier: 1,
                    offset: 0,
                }),
                toughness: Some(StatExpr {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 1,
                        kind: 0,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    multiplier: 1,
                    offset: 0,
                }),
            },
        )],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(DynamicStats {
            power: Some(StatExpr {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 1,
                    kind: 0,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                multiplier: 1,
                offset: 0,
            }),
            toughness: Some(StatExpr {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 1,
                    kind: 0,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                multiplier: 1,
                offset: 0,
            }),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
