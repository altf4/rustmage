// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Discard,
                players: PlayerSet::Opponent,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Skip(1),
            Instruction::CounterRelated,
        ],
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
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::FLYING.union(Keywords::TRAMPLE),
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Base,
            fragments: &[StaticAbilityFragment::LiveBase(
                0,
                LiveStaticEffect {
                    recipients: LiveStatRecipients::Creatures {
                        relation: 1,
                        other: false,
                        enchanted_by_controller: false,
                    },
                    condition: None,
                    power: None,
                    toughness: Some(LiveStatValue::Fixed(1)),
                    bonus_power: 0,
                    bonus_toughness: 0,
                    keywords: Keywords::NONE,
                    cannot_block_subtype: None,
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                    Keywords::FLYING.union(Keywords::TRAMPLE),
                )],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Base,
                fragments: &[StaticAbilityFragment::LiveBase(
                    0,
                    LiveStaticEffect {
                        recipients: LiveStatRecipients::Creatures {
                            relation: 1,
                            other: false,
                            enchanted_by_controller: false,
                        },
                        condition: None,
                        power: None,
                        toughness: Some(LiveStatValue::Fixed(1)),
                        bonus_power: 0,
                        bonus_toughness: 0,
                        keywords: Keywords::NONE,
                        cannot_block_subtype: None,
                    },
                )],
            },
        ],
        live_static_effects: &[LiveStaticEffect {
            recipients: LiveStatRecipients::Creatures {
                relation: 1,
                other: false,
                enchanted_by_controller: false,
            },
            condition: None,
            power: None,
            toughness: Some(LiveStatValue::Fixed(1)),
            bonus_power: 0,
            bonus_toughness: 0,
            keywords: Keywords::NONE,
            cannot_block_subtype: None,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
