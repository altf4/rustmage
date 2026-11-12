pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SURVIVAL,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseSourceCounterKind(&[
                CounterKind::Flying,
                CounterKind::Lifelink,
                CounterKind::PlusOne,
            ])],
        },
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                kinds: 1,
                relation: CastRelation::You,
                subject: DepartureSubject::SelfOrOtherMatching,
                destinations: &[crate::engine::Zone::Graveyard],
                counters: DepartureCounters::Any,
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::PutEventCountersOnTargets],
        },
    ],
    ..CardRules::EMPTY
};
