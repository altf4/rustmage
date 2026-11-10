use crate::engine::Zone;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        warp: Some(&WarpSpec::fixed(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
        })),
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                kinds: 1,
                relation: CastRelation::You,
                destinations: &[Zone::Graveyard],
                nontoken: true,
                counters: DepartureCounters::Kind(CounterKind::PlusOne),
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5241),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
