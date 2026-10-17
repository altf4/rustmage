// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::JobSelect { token: Card(5328) }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 4,
            alternatives: &[],
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
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::AttachSource],
        sorcery: true,
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
    keywords: Keywords::NONE,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::AttachedSubtype("Artificer"),
            StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Attached,
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
                    stats: Stats {
                        power: 2,
                        toughness: 2,
                    },
                },
            ),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[
                StaticAbilityFragment::AttachedSubtype("Artificer"),
                StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Attached,
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
                        stats: Stats {
                            power: 2,
                            toughness: 2,
                        },
                    },
                ),
            ],
        }],
        attached_subtype: Some("Artificer"),
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Attached,
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
            stats: Stats {
                power: 2,
                toughness: 2,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
