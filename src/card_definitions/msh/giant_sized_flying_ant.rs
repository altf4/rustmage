// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Tap target nonland permanent.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                })),
                program: &[Instruction::Tap],
            },
            SpellMode {
                text: "Untap target nonland permanent.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                })),
                program: &[Instruction::Untap],
            },
        ])],
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
    keywords: Keywords::FLYING.union(Keywords::FLASH),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
