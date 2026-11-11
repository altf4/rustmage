// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupBonus {
            filters: &[CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: Some(true),
                artifact: false,
            }],
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Pollen-Shield Hare // Hare Raising"),
        adventure_spell: Some(Card::from_catalogue_id(5559)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
