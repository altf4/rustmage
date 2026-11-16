// Pinned Oracle and XMage: choose one complete mode at announcement.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target creature with power 4 or greater.",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::PowerAtLeast(4),
            }),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Draw a card.",
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ]),
    ..CardRules::EMPTY
};
