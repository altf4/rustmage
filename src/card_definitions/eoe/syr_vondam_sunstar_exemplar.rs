const DEATH_OR_EXILE: &[crate::engine::Zone] =
    &[crate::engine::Zone::Graveyard, crate::engine::Zone::Exile];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::VIGILANCE.union(Keywords::MENACE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                kinds: 1,
                relation: CastRelation::You,
                subject: DepartureSubject::Other,
                destinations: DEATH_OR_EXILE,
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                subject: DepartureSubject::SelfOnly,
                destinations: DEATH_OR_EXILE,
                minimum_power: Some(4),
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::Destroy],
        },
    ],
    ..CardRules::EMPTY
};
