// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: 0,
                other: true,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Subtype("Elf"),
        })),
        program: &[Instruction::ReturnToHand],
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
    keywords: Keywords(0),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::StaticModifier(
            0,
            StaticModifier {
                own_turn: false,
                requires_life_gain: false,
                filter: CreatureFilter {
                    relation: 0,
                    other: true,
                    subtype: Some("Elf"),
                    keyword: Keywords(0),
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            },
        )],
    }],
    extras: &CardExtras {
        static_modifiers: &[StaticModifier {
            own_turn: false,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: true,
                subtype: Some("Elf"),
                keyword: Keywords(0),
                attacking: false,
                tokens: None,
                artifact: false,
            },
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::StaticModifier(
                0,
                StaticModifier {
                    own_turn: false,
                    requires_life_gain: false,
                    filter: CreatureFilter {
                        relation: 0,
                        other: true,
                        subtype: Some("Elf"),
                        keyword: Keywords(0),
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    stats: Stats {
                        power: 1,
                        toughness: 1,
                    },
                },
            )],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
