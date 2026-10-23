// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: usize::MAX,
        }),
        &[
            Instruction::BlinkGroupTappedUntilEndStep,
            Instruction::PreventPlayerDamageUntilNextTurn,
            Instruction::ExileResolvingSpell,
        ],
    ),
    ..CardRules::EMPTY
};
