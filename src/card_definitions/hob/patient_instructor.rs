// Complete printed behavior from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Recruit { token: Card(5241) }],
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
