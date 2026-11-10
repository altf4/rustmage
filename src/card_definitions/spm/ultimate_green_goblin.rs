pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Discard(PlayerSet::Controller, 1),
            Instruction::CreateTokens {
                token: Card(5302),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 1, 1, 0, 0],
                generic: 1,
            },
            ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 1,
            },
        ],
        mayhem_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        }),
        mayhem_cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
