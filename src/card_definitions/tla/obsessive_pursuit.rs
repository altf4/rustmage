// Direct typed behavior from all printed clauses and the complete pinned XMage class.
const CLUE: &[Instruction] = &[
    Instruction::LoseLife(PlayerSet::Controller, 1),
    Instruction::CreateTokens {
        token: Card(5296),
        amount: 1,
        tapped: false,
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: CLUE,
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: CLUE,
        },
        TriggeredAbility {
            condition: TriggerCondition::AttackDeclared {
                minimum: 1,
                power_min: None,
                guard: StepGuard::Always,
            },
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Attacking,
            }),
            program: &[
                Instruction::PutCountersScaled {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::Stat(StatValue::PermanentsSacrificedThisTurn),
                },
                Instruction::If {
                    guard: StepGuard::Expression {
                        value: StatValue::PermanentsSacrificedThisTurn,
                        minimum: Some(3),
                        maximum: None,
                    },
                    skip: 1,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::LIFELINK,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
