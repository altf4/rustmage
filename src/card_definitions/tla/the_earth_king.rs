// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5708),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreaturesAttackDeclared {
                minimum: 1,
                power_min: Some(4),
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SearchScaled {
                spec: SearchSpec {
                    filter: SearchFilter::BasicLand,
                    maximum: 1,
                    destination: SearchDestination::Battlefield,
                    tapped: true,
                    reveal: false,
                },
                maximum: AmountValue::DeclaredAttackerCount,
            }],
        },
    ],
    ..CardRules::EMPTY
};
