// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfBlighted { skip: 2 },
            Instruction::ExileTopPlay {
                amount: 3,
                next_turn: true,
            },
            Instruction::Skip(1),
            Instruction::ExileTopPlay {
                amount: 2,
                next_turn: true,
            },
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
