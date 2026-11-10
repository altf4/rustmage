pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CounterPlacedByYou {
            kind: CounterKind::PlusOne,
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 1,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayOncePerTurn { skip: 2 },
            Instruction::RecordOncePerTurn,
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::CounterPlacementAmount),
        ],
    }],
    ..CardRules::EMPTY
};
