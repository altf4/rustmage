pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesPlotted,
        targets: one(TargetRule::Any),
        program: &[Instruction::Damage(2)],
    }],
    extras: &CardExtras {
        plot_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
