// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
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
                power: 1,
                toughness: 0,
            },
            keywords: Keywords(1),
        },
    }),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::AttachmentStats(Stats {
                    power: 1,
                    toughness: 0,
                }),
                StaticAbilityFragment::AttachmentKeywords(Keywords(1)),
            ],
        },
    ],
    extras: &CardExtras {
        aura: Some(AuraSpec {
            kinds: 1,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 1,
                    toughness: 0,
                },
                keywords: Keywords(1),
            },
        }),
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32))],
            },
            StaticAbilityGroup {
                line: 3,
                first_layer: AbilityLayer::Ability,
                fragments: &[
                    StaticAbilityFragment::AttachmentStats(Stats {
                        power: 1,
                        toughness: 0,
                    }),
                    StaticAbilityFragment::AttachmentKeywords(Keywords(1)),
                ],
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
