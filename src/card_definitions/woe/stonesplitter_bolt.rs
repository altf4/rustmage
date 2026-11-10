pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1 | 64,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::IfBargained { skip: 2 },
            Instruction::DamageScaled(AmountValue::Multiple {
                value: &AmountValue::ChosenX,
                factor: 2,
            }),
            Instruction::Skip(1),
            Instruction::DamageScaled(AmountValue::ChosenX),
        ],
    ),
    extras: &CardExtras {
        bargain_reduction: Some(0),
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
