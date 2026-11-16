// Oracle and official Star Trek release notes; no pinned XMage class exists.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 1 | 16,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        entry_copy_options: Some(&EntryCopyOptions {
            source: EntryCopySource::BattlefieldWithCounters,
            exceptions: CopyExceptions {
                remove_legendary: true,
                ..CopyExceptions::NONE
            },
            ..EntryCopyOptions::DEFAULT
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
