// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::Draw(PlayerSet::Controller, 2),
                Instruction::Discard(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::SelfAttacks,
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: true,
                    }),
                    minimum: Some(7),
                    maximum: None,
                },
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::May { skip: 1 },
                Instruction::CreateTokens {
                    token: Card(5334),
                    amount: 1,
                    tapped: false,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
