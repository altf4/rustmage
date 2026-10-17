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
    keywords: Keywords::NONE,
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: -5,
                toughness: -3,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::TargetCostReduction(
                0,
                TargetSpellReduction {
                    rule: TargetRule::Filtered {
                        base: &TargetRule::Creature,
                        predicate: &TargetPredicate::Keyword(Keywords::FLYING),
                    },
                    discount: ManaCost {
                        colored: [0, 0, 0, 0, 0, 0],
                        generic: 1,
                    },
                },
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::AttachmentStats(Stats {
                power: -5,
                toughness: -3,
            })],
        },
    ],
    extras: &CardExtras {
        aura: Some(AuraSpec {
            kinds: 1,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: -5,
                    toughness: -3,
                },
                keywords: Keywords::NONE,
            },
        }),
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Rule,
                fragments: &[StaticAbilityFragment::TargetCostReduction(
                    0,
                    TargetSpellReduction {
                        rule: TargetRule::Filtered {
                            base: &TargetRule::Creature,
                            predicate: &TargetPredicate::Keyword(Keywords::FLYING),
                        },
                        discount: ManaCost {
                            colored: [0, 0, 0, 0, 0, 0],
                            generic: 1,
                        },
                    },
                )],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Modify,
                fragments: &[StaticAbilityFragment::AttachmentStats(Stats {
                    power: -5,
                    toughness: -3,
                })],
            },
        ],
        target_cost_reduction: Some(&TargetSpellReduction {
            rule: TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Keyword(Keywords::FLYING),
            },
            discount: ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 1,
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
