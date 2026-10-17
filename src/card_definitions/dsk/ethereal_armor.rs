// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(0),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords(512),
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Attached,
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 32,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    stats: Stats {
                        power: 1,
                        toughness: 1,
                    },
                },
            ),
            StaticAbilityFragment::AttachmentKeywords(Keywords(512)),
        ],
    }],
    extras: &CardExtras {
        aura: Some(AuraSpec {
            kinds: 1,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 0,
                    toughness: 0,
                },
                keywords: Keywords(512),
            },
        }),
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Attached,
                        value: StatValue::Count(CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 32,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: false,
                        }),
                        stats: Stats {
                            power: 1,
                            toughness: 1,
                        },
                    },
                ),
                StaticAbilityFragment::AttachmentKeywords(Keywords(512)),
            ],
        }],
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Attached,
            value: StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 32,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            }),
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
