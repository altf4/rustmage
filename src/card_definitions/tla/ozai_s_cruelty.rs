// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: false,
        }),
        &[
            Instruction::Damage(2),
            Instruction::Discard(PlayerSet::Targeted, 2),
        ],
    ),
    ..CardRules::EMPTY
};
