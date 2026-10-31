// Count only the exact incarnations that actually entered graveyards in this mill event.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::Mill(PlayerSet::Each, MillAmount::ChosenX),
            Instruction::CreateTokensScaled {
                token: Card(5258),
                amount: AmountValue::MilledCreatures,
                tapped: true,
            },
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
