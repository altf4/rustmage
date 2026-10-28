// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PermanentToGraveyard {
                kinds: 65,
                relation: CastRelation::You,
                exclude_source_kinds: 65,
            },
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[
                Instruction::Damage(1),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::AnyNoncombatSources,
                combat_only: false,
                opponent_only: true,
                one_or_more: true,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
