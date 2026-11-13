// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target creature with no counters on it.",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::NoCounters,
            }),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Remove up to three counters from target creature.",
            targets: one(TargetRule::Creature),
            program: &[Instruction::RemoveUpToTargetCounters(3)],
        },
    ]),
    ..CardRules::EMPTY
};
