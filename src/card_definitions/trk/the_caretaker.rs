// Oracle and official Star Trek release notes reviewed; XMage class absent at the pinned revision.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 6),
    keywords: Keywords::VIGILANCE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: generic(2), subject: CounterSubject::Related,
                exile: false, unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::Any,
                guard: StepGuard::Expression {
                    value: StatValue::DilemmasFacedThisTurn,
                    minimum: Some(1), maximum: None,
                },
            },
            targets: one(TargetRule::Player { opponent_only: true }),
            program: &[Instruction::ExileTargetLibraryUntilNonlandCastFree],
        },
    ],
    ..CardRules::EMPTY
};
