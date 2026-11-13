pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksWithOthers,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [1, 1, 0, 0, 0, 0],
                generic: 1,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 1,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
