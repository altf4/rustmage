pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::IfTargetManaValueAtMost {
                maximum: 3,
                skip: 3,
            },
            Instruction::Exile,
            Instruction::Surveil(2),
            Instruction::Skip(1),
            Instruction::Exile,
        ],
    ),
    ..CardRules::EMPTY
};
