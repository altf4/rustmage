// Authored from pinned Oracle and official release notes; no XMage card class exists.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CreatureEnters {
                relation: CastRelation::You,
                other: false,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Scry(1),
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Plan,
                    amount: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::CounterThresholdCrossed {
                kind: CounterKind::Plan,
                threshold: 4,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::SacrificeSource,
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::PutPlayerCreatureCounters {
                    players: PlayerSet::Controller,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
