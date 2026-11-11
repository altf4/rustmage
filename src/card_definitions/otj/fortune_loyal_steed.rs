pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Scry(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacksWhileSaddled,
            targets: TargetPlan::NONE,
            program: &[Instruction::ScheduleStepThisTurn {
                step: TriggerStep::EndCombat,
                relation: CastRelation::Any,
                ability: 2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseSaddleRiders(
                SaddleRiderEffect::BlinkSourceAndUpToOne,
            )],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::BecomeSaddled { minimum: 1 }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
