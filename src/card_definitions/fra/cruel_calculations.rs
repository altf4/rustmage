// Direct Oracle and pinned XMage behavior; gated pending full validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: false,
        }),
        &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::TargetPlayerLibraryToGraveyardThisTurn,
        )],
    ),
    ..CardRules::EMPTY
};
