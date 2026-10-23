// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfSneakPaid { skip: 2 },
            Instruction::CreateTokensAttacking {
                token: Card(5359),
                amount: 3,
            },
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: Card(5359),
                amount: 3,
                tapped: false,
            },
        ],
    ),
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
