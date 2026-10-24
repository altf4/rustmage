// Life X is an additional cost; the printed mana cost and mana value stay fixed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ReturnGraveyardManaValue {
            kinds: 17,
            value: AmountValue::ChosenX,
        }],
    ),
    extras: &CardExtras {
        life_x_cost: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
