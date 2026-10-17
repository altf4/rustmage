// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::CreateRole {
            token: Card(5330),
            subject: Subject::Targets,
        }],
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
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::EnchantedModifier(
            0,
            EnchantedModifier {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
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
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::EnchantedModifier(
                0,
                EnchantedModifier {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
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
        enchanted_modifiers: &[EnchantedModifier {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
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
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
