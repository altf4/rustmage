// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::If {
                    guard: StepGuard::Expression {
                        value: StatValue::ControllerHandSize,
                        minimum: Some(1),
                        maximum: None,
                    },
                    skip: 2,
                },
                Instruction::Discard(PlayerSet::Controller, 1),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayFromOutsideOwnHand(
                &TriggerCondition::SpellCastOrLandPlayed,
            ),
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 1)],
        },
    ],
    ..CardRules::EMPTY
};
