// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::Any,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayDrawForPlayer {
                    player: PlayerSet::Active,
                    skip: 3,
                },
                Instruction::Draw(PlayerSet::Active, 1),
                Instruction::ProhibitAttacksThisTurn(PlayerSet::Active),
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Peace,
                    amount: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::Upkeep,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::Expression {
                    value: StatValue::SourceCounters(CounterKind::Peace),
                    minimum: Some(12),
                    maximum: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::WinGame],
        },
    ],
    ..CardRules::EMPTY
};
