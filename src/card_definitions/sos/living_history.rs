// Direct typed Oracle behavior; full XMage source pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5350),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::Expression {
                    value: StatValue::CardLeftGraveyardThisTurn,
                    minimum: Some(1),
                    maximum: None,
                },
                event: &TriggerCondition::AttackDeclared {
                    minimum: 1,
                    power_min: None,
                    guard: StepGuard::Always,
                },
            },
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Attacking,
            }),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 0,
            }],
        },
    ],
    ..CardRules::EMPTY
};
