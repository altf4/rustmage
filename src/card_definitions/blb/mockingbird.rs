pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    extras: &CardExtras {
        x_symbols: 1,
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
        entry_copy_options: Some(&EntryCopyOptions {
            source: EntryCopySource::BattlefieldAtMostManaSpent,
            exceptions: CopyExceptions {
                bird: true,
                ..CopyExceptions::NONE
            },
            ..EntryCopyOptions::DEFAULT
        }),
        entry_copy_keywords: Keywords::FLYING,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
