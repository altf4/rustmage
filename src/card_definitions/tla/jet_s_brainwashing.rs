// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BLOCK,
            },
            Instruction::IfKicked { skip: 3 },
            Instruction::GainControlUntilEndTurn,
            Instruction::Untap,
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
            Instruction::CreateTokens {
                token: Card(5296),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    extras: &CardExtras {
        kicker_cost: Some(ManaCost {
            colored: [0; 6],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
