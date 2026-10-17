// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::ChosenX,
                minimum: 10,
                skip: 1,
            },
            Instruction::ShuffleGraveyardIntoLibrary,
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::ChosenX),
            Instruction::IfAmountAtLeast {
                value: AmountValue::ChosenX,
                minimum: 10,
                skip: 2,
            },
            Instruction::UntapLands(5),
            Instruction::NoMaximumHandSize,
            Instruction::ExileResolvingSpell,
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
