// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[
            // Exclude lands, instants and sorceries from the revealed hand.
            Instruction::RevealHandDiscard {
                excluded_kinds: 2 | 4 | 8,
            },
            Instruction::EnsureJaceToken,
            Instruction::EmpowerJace(1),
        ],
    ),
    ..CardRules::EMPTY
};
