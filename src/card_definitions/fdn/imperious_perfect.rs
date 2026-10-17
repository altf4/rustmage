// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::CreateTokens {
            token: Card(5310),
            amount: 1,
            tapped: false,
        }],
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
