pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::RequireAllAttacksUntilNextTurn],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 3, 0, 0,
            ]))],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 5)],
        },
    ],
    ..CardRules::EMPTY
};
