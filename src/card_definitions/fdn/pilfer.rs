// Direct typed behavior; paired Oracle and executable XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: true,
            },
            min: 1,
            max: 1,
        }),
        &[Instruction::RevealHandDiscard { excluded_kinds: 2 }],
    ),
    ..CardRules::EMPTY
};
