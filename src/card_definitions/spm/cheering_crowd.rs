pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfSourceCanReceiveCounters { skip: 3 },
            Instruction::MayForPlayer {
                player: PlayerSet::Active,
                skip: 2,
            },
            Instruction::PaySourceCounterPlacement {
                player: PlayerSet::Active,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::AddColorlessScaledFor(PlayerSet::Active, AmountValue::SourceTotalCounters),
        ],
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
