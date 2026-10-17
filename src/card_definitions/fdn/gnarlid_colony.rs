// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
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
    keywords: Keywords(0),
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::CounterKeyword(
            0,
            CounterKind::PlusOne,
            Keywords(256),
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::CounterKeyword(
                0,
                CounterKind::PlusOne,
                Keywords(256),
            )],
        }],
        kicker_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 2,
        }),
        kicked_entry_counters: 2,
        counter_keyword_grants: &[(CounterKind::PlusOne, Keywords(256))],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
