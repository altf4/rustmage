// Direct whole-card behavior from frozen Oracle and the full pinned Java effect.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ExileHandDraw,
            Instruction::GrantExiledResultPlay { next_turn: true },
        ],
    ),
    ..CardRules::EMPTY
};
