// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(32),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 3,
                toughness: 2,
            },
            keywords: Keywords(2),
        },
    }),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::AttachmentStats(Stats {
                    power: 3,
                    toughness: 2,
                }),
                StaticAbilityFragment::AttachmentKeywords(Keywords(2)),
            ],
        },
    ],
    extras: &CardExtras {
        aura: Some(AuraSpec {
            kinds: 1,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 3,
                    toughness: 2,
                },
                keywords: Keywords(2),
            },
        }),
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Ability,
                fragments: &[
                    StaticAbilityFragment::AttachmentStats(Stats {
                        power: 3,
                        toughness: 2,
                    }),
                    StaticAbilityFragment::AttachmentKeywords(Keywords(2)),
                ],
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
