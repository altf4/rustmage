// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::DamagePlayers(PlayerSet::Opponent, 3),
                Instruction::GainLife(PlayerSet::Controller, 3),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::DiscardAnyDraw {
                players: PlayerSet::Controller,
                bonus: 1,
                required_all: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
