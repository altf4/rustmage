pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::VIGILANCE.union(Keywords::LIFELINK),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::May { skip: 1 },
            Instruction::ExchangeLifeTotalsWithTarget {
                draw_controller_loss: true,
            },
        ],
    }],
    ..CardRules::EMPTY
};
