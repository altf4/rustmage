// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: Keywords::CHANGELING,
    extras: &CardExtras {
        has_convoke: true,
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        entry_copy_keywords: Keywords::CHANGELING,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
