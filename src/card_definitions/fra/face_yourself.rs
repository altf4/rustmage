// Direct Oracle and pinned XMage rules; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: false,
        }),
        &[Instruction::CopyTargetPlayersCreaturesWithConditionalSacrifice],
    ),
    ..CardRules::EMPTY
};
