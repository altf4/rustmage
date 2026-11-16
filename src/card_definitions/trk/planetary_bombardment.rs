// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::CreaturesDiedThisTurn),
                minimum: 1,
                skip: 2,
            },
            Instruction::Damage(4),
            Instruction::Skip(1),
            Instruction::Damage(2),
        ],
    ),
    ..CardRules::EMPTY
};
