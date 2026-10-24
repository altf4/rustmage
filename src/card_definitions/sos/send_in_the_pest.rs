// Direct typed Oracle behavior; full XMage source pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Discard(PlayerSet::Opponent, 1),
            Instruction::CreateTokens {
                token: Card(5348),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
