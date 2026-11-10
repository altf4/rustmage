// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: false,
        }),
        &[Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(4))],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(903)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
