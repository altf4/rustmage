// Authored from pinned Oracle and complete XMage source; awaiting rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[
            Instruction::Damage(2),
            Instruction::RevealHandDiscard { excluded_kinds: 2 },
        ],
    ),
    ..CardRules::EMPTY
};
