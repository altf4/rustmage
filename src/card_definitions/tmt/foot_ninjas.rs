// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    extras: &CardExtras {
        sneak_costs: &[
            ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 3,
            },
            ManaCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 3,
            },
        ],
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 4,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
