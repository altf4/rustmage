// Direct typed rules from pinned Oracle and XMage; gated pending tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::SourceGraveyardBonus {
            kind: 0,
            minimum: 7,
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    extras: &CardExtras {
        enters_prepared: true,
        prepared_spell: Some(Card(5452)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
