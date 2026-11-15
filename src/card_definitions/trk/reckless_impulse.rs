// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileTopPlay {
            amount: 2,
            next_turn: true,
        }],
    ),
    ..CardRules::EMPTY
};
