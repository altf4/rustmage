// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
            alternatives: &[ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 2,
            }],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Surveil(1)],
        sorcery: false,
        once_per_turn: false,
    }],
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
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(
            0,
            ScalingModifier {
                subject: Subject::Source,
                value: StatValue::AtLeast {
                    value: &StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    minimum: 7,
                },
                stats: Stats {
                    power: 3,
                    toughness: 0,
                },
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: StatValue::AtLeast {
                        value: &StatValue::Count(CountFilter {
                            zone: CountZone::Graveyard,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: true,
                        }),
                        minimum: 7,
                    },
                    stats: Stats {
                        power: 3,
                        toughness: 0,
                    },
                },
            )],
        }],
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: StatValue::AtLeast {
                value: &StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                }),
                minimum: 7,
            },
            stats: Stats {
                power: 3,
                toughness: 0,
            },
        }],
        cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 1, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 2,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
