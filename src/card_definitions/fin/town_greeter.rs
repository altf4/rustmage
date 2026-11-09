// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(2),
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 1,
                },
                destination: MilledDestination::HandAndCountSubtype("Town"),
            }),
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
