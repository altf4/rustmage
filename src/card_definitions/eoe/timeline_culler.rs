pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::HASTE,
    extras: &CardExtras {
        warp: Some(&WarpSpec {
            life: 2,
            from_graveyard: true,
            ..WarpSpec::fixed(ManaCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 0,
            })
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
