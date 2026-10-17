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
                power: 2,
                toughness: 2,
            },
            keywords: Keywords::LIFELINK.union(Keywords::TRAMPLE),
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 2,
                toughness: 2,
            }),
            StaticAbilityFragment::AttachmentKeywords(Keywords::LIFELINK.union(Keywords::TRAMPLE)),
        ],
    }],
    extras: &CardExtras {
        aura: Some(AuraSpec {
            kinds: 1,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 2,
                    toughness: 2,
                },
                keywords: Keywords::LIFELINK.union(Keywords::TRAMPLE),
            },
        }),
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[
                StaticAbilityFragment::AttachmentStats(Stats {
                    power: 2,
                    toughness: 2,
                }),
                StaticAbilityFragment::AttachmentKeywords(
                    Keywords::LIFELINK.union(Keywords::TRAMPLE),
                ),
            ],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
