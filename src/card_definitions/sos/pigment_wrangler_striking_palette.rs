// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    }],
    extras: &CardExtras {
        prepared_spell: Some(Card(5375)),
        enters_prepared: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
