// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(10, 10),
    keywords: Keywords::PROTECTION_EVERYTHING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::PROTECTION_EVERYTHING,
        )],
    }],
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::RevealShuffleSelf),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
