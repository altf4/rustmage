// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfMayhemPaid {
                paid: true,
                skip: 2,
            },
            Instruction::GrowPlayerCreatures {
                players: PlayerSet::Opponent,
                power: -2,
                toughness: -2,
            },
            Instruction::Skip(1),
            Instruction::GrowPlayerCreatures {
                players: PlayerSet::Each,
                power: -2,
                toughness: -2,
            },
        ],
    ),
    extras: &CardExtras {
        mayhem_cost: Some(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
