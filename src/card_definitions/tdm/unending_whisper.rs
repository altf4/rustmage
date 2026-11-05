// Complete Oracle and pinned XMage: the same draw uses either cast route.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    extras: &CardExtras {
        harmonize_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
