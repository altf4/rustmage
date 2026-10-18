// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(11, 11),
    keywords: Keywords::TRAMPLE.union(Keywords::INDESTRUCTIBLE),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::TRAMPLE.union(Keywords::INDESTRUCTIBLE),
        )],
    }],
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::RevealShuffleSelf),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
