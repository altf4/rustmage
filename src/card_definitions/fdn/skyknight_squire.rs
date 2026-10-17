// Gated core consumer: its enter trigger is still pending; this is not admitted.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
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
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::SourceCounterSubtype {
            kind: CounterKind::PlusOne,
            minimum: 3,
            subtype: "Knight",
            keywords: Keywords::FLYING,
        }],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::SourceCounterSubtype {
                kind: CounterKind::PlusOne,
                minimum: 3,
                subtype: "Knight",
                keywords: Keywords::FLYING,
            }],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
