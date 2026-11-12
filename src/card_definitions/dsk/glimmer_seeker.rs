pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 1,
                        exclude: 0,
                        subtype: Some("Glimmer"),
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 2,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5294), // Glimmer Token
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
