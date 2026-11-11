// Direct typed behavior from the paired Oracle and XMage sources.
const PAYOFF: &[Instruction] = &[
    Instruction::GainLife(PlayerSet::Controller, 1),
    Instruction::Draw(PlayerSet::Controller, 1),
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: PAYOFF,
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfExiledWhileCrafting,
            targets: TargetPlan::NONE,
            program: PAYOFF,
        },
    ],
    ..CardRules::EMPTY
};
