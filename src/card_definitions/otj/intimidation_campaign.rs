pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLife(PlayerSet::Opponent, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::CommittedCrime,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayIfSourceOnBattlefield { skip: 1 },
                Instruction::ReturnSourceToHand(TargetZone::Battlefield),
            ],
        },
    ],
    ..CardRules::EMPTY
};
