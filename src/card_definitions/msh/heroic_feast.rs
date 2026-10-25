// Direct typed behavior from the full frozen Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5246),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::LifeGained {
                first_only: false,
                own_turn: false,
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 0,
                max: usize::MAX,
            }),
            program: &[
                Instruction::UpToTargets(AmountValue::LifeEventAmount),
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
