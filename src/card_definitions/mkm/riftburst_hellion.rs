pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 7),
    keywords: Keywords::REACH,
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 4,
            },
            alternatives: &[
                ManaCost {
                    colored: [0, 0, 0, 1, 1, 0],
                    generic: 4,
                },
                ManaCost {
                    colored: [0, 0, 0, 0, 2, 0],
                    generic: 4,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
