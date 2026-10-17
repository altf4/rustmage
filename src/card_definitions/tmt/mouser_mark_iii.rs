// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
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
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(
            0,
            CombatRestriction {
                subject: Subject::Source,
                attack: true,
                block: false,
                unless: Some(StepGuard::Expression {
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
                    minimum: Some(1),
                    maximum: None,
                }),
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CombatRestriction(
                0,
                CombatRestriction {
                    subject: Subject::Source,
                    attack: true,
                    block: false,
                    unless: Some(StepGuard::Expression {
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
                        minimum: Some(1),
                        maximum: None,
                    }),
                },
            )],
        }],
        combat_restriction: Some(CombatRestriction {
            subject: Subject::Source,
            attack: true,
            block: false,
            unless: Some(StepGuard::Expression {
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
                minimum: Some(1),
                maximum: None,
            }),
        }),
        cost_alternatives: &[ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
