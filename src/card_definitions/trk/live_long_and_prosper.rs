// Complete pinned Oracle and XMage LiveLongAndProsper class.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DoubleControllerLifeTotal,
            Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::ControllerHandSize),
            ),
            Instruction::NoMaximumHandSizeUntilNextTurn,
        ],
    ),
    extras: &CardExtras {
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
