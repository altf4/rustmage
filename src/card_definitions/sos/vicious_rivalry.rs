// Life X is an additional cost; the printed mana cost and mana value stay fixed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DestroyManaValue {
            kinds: 17,
            maximum: AmountValue::ChosenX,
        }],
    ),
    extras: &CardExtras {
        life_x_cost: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
