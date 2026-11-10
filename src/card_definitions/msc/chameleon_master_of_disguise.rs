pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        entry_copy_options: Some(&EntryCopyOptions {
            name: Some("Chameleon, Master of Disguise"),
            ..EntryCopyOptions::DEFAULT
        }),
        mayhem_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
