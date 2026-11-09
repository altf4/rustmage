pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[Instruction::Mill(
            PlayerSet::Targeted,
            MillAmount::HalfLibraryRoundedDown,
        )],
    ),
    ..CardRules::EMPTY
};
