// Direct Oracle and pinned XMage behavior; gated pending integration tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep, relation: CastRelation::You, guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Surveil(1),
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Graveyard, relation: 0, kind: 0, exclude: 0,
                        subtype: None, basic: false, other: false, cards_only: true,
                    }),
                    minimum: Some(7), maximum: None,
                },
                skip: 3,
            },
            Instruction::SacrificeSource,
            Instruction::DamagePlayers(PlayerSet::Opponent, 2),
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
