// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Grow {
            subject: Subject::Targets,
            power: 4,
            toughness: 4,
        }],
    ),
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
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::TargetCostReduction(
            0,
            TargetSpellReduction {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: 4294967295,
                    },
                    predicate: &TargetPredicate::Subtype("Spider"),
                },
                discount: ManaCost {
                    colored: [0, 0, 0, 0, 0, 0],
                    generic: 1,
                },
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::TargetCostReduction(
                0,
                TargetSpellReduction {
                    rule: TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Battlefield,
                            kinds: 0,
                            relation: -1,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: 4294967295,
                        },
                        predicate: &TargetPredicate::Subtype("Spider"),
                    },
                    discount: ManaCost {
                        colored: [0, 0, 0, 0, 0, 0],
                        generic: 1,
                    },
                },
            )],
        }],
        target_cost_reduction: Some(&TargetSpellReduction {
            rule: TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: 4294967295,
                },
                predicate: &TargetPredicate::Subtype("Spider"),
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
