// Complete pinned Oracle and XMage obligations; branch before the counter attempt.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Spell),
        &[
            Instruction::IfTargetSpellManaValueAtMost {
                maximum: 2,
                skip: 3,
            },
            Instruction::Counter,
            Instruction::Recruit { token: Card(5241) },
            Instruction::Skip(1),
            Instruction::Counter,
        ],
    ),
    ..CardRules::EMPTY
};
