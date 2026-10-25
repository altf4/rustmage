// Extort is untargeted. The paid attack rule changes assignment, never power.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Any,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayManaAlternatives {
                    costs: &[
                        ManaCost {
                            colored: [1, 0, 0, 0, 0, 0],
                            generic: 0,
                        },
                        ManaCost {
                            colored: [0, 0, 1, 0, 0, 0],
                            generic: 0,
                        },
                    ],
                    skip: 1,
                },
                Instruction::DrainOpponents(1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::AttackDeclared {
                minimum: 1,
                power_min: None,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayControllerLife { amount: 2, skip: 2 },
                Instruction::LoseLife(PlayerSet::Controller, 2),
                Instruction::CombatDamageByGreaterToughnessThisTurn,
            ],
        },
    ],
    ..CardRules::EMPTY
};
