// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        &[Instruction::DiscardThenPutLand {
            players: PlayerSet::Targeted,
            amount: 2,
            tapped: true,
        }],
    ),
    ..CardRules::EMPTY
};
