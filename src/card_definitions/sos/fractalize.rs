// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::SetCreatureForm {
            types: &["Fractal"],
            colors: 2 | 16,
            base: AmountValue::Sum(&[AmountValue::ChosenX, AmountValue::Fixed(1)]),
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
