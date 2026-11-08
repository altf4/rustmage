// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::IfBlighted { skip: 2 },
            Instruction::Damage(4),
            Instruction::Skip(1),
            Instruction::Damage(2),
        ],
    ),
    extras: &CardExtras {
        blight: Some(BlightSpec {
            amount: 1,
            normal_modes: 0,
            paid_modes: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
