// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseWheel { draw: 7 },
            Instruction::DiscardWholeHands(PlayerSet::Wheelers),
            Instruction::Draw(PlayerSet::Wheelers, 7),
        ],
    ),
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
