// Whole-card typed Oracle behavior; full XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::DamageForExcess(AmountValue::ColorsSpent),
            Instruction::ExileExcessPlay { next_turn: true },
        ],
    ),
    ..CardRules::EMPTY
};
