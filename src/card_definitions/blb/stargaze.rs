pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::ChosenX,
                    filter: LibrarySelectionFilter::Any,
                    optional: false,
                    reveal: false,
                    remainder: LibraryRemainder::Graveyard,
                },
                look: LibraryLook::Amount(AmountValue::Multiple {
                    value: &AmountValue::ChosenX,
                    factor: 2,
                }),
                destination: LibraryPickDestination::Hand,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::ChosenX),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
