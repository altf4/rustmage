pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(3)),
            Instruction::ExileTargetGraveyardForWaterbend,
        ],
    }],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 2, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 2, 1, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 3, 0, 0, 0, 0],
                generic: 2,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
