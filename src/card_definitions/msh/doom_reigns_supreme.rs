// Complete typed Plan threshold and sacrifice-success reflexive programs.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SubtypeEntersYou("Villain"),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLife(PlayerSet::Opponent, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
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
                threshold: 5,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSourceReflexive { ability: 2 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::ExileTargetLibraryTopCastFree {
                amount: 5,
                maximum_casts: 2,
            }],
        },
    ],
    ..CardRules::EMPTY
};
