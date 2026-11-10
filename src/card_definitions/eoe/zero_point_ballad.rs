pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DestroyCreaturesByToughness {
                maximum: AmountValue::ChosenX,
                remember: true,
            },
            Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::ChosenX),
            Instruction::IfAmountAtLeast {
                value: AmountValue::ChosenX,
                minimum: 6,
                skip: 1,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnRememberedCreature,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
