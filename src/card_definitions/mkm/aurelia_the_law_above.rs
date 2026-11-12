pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING
        .union(Keywords::VIGILANCE)
        .union(Keywords::HASTE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerAttacksWithAtLeast(3),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerAttacksWithAtLeast(5),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::DamagePlayers(PlayerSet::Opponent, 3),
                Instruction::GainLife(PlayerSet::Controller, 3),
            ],
        },
    ],
    ..CardRules::EMPTY
};
