pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExploreRepeated {
            subject: Subject::Source,
            amount: AmountValue::ChosenX,
        }],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
