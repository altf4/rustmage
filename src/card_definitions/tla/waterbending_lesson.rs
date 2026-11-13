// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 3),
            Instruction::MayPayWaterbend {
                amount: 2,
                unpaid_skip: 1,
            },
            Instruction::Skip(1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
