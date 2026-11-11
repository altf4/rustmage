pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesWithCounter {
            relation: CastRelation::You,
            kind: CounterKind::PlusOne,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Creature),
        program: &[Instruction::MoveCountersFromSource {
            kind: CounterKind::PlusOne,
            amount: AmountValue::Fixed(1),
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        fixed_entry_counters: &[(CounterKind::PlusOne, 2)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
