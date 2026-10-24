// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 1,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    minimum: Some(2),
                    maximum: None,
                },
                skip: 2,
            },
            Instruction::CreateTokens {
                token: Card(5312),
                amount: 1,
                tapped: true,
            },
            Instruction::Skip(1),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
        ],
    }],
    ..CardRules::EMPTY
};
