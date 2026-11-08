// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Threshold {
                value: StepValue::Hand,
                minimum: None,
                maximum: Some(3),
            },
            event: &TriggerCondition::SelfEnters,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawToHandSize {
            players: PlayerSet::Controller,
            size: 4,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(
            0,
            ScalingModifier {
                subject: Subject::Source,
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 16,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: true,
                    cards_only: false,
                }),
                stats: Stats {
                    power: 1,
                    toughness: 0,
                },
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 16,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: true,
                        cards_only: false,
                    }),
                    stats: Stats {
                        power: 1,
                        toughness: 0,
                    },
                },
            )],
        }],
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 16,
                exclude: 0,
                subtype: None,
                basic: false,
                other: true,
                cards_only: false,
            }),
            stats: Stats {
                power: 1,
                toughness: 0,
            },
        }],
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Stat {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 16,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
                factor: 1,
            },
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
