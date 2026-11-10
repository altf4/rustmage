pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    enters: Counters {
        plus_one: 2,
        ..CardRules::EMPTY.enters
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::MoveCountersFromSource {
            kind: CounterKind::PlusOne,
            amount: AmountValue::Fixed(1),
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
            kinds: 1,
            relation: CastRelation::You,
            modified: true,
            ..DepartureRule::ANY
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
