// Frozen Oracle and official notes; direct class absent from pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CreatureCardsEnterYourGraveyard,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Controller, 1),
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Plan,
                    amount: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::CounterThresholdCrossed {
                kind: CounterKind::Plan,
                threshold: 3,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::SacrificeSource,
                Instruction::CreateTokens {
                    token: Card(5388),
                    amount: 3,
                    tapped: false,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
