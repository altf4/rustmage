// Complete Oracle/XMage: qualify the actual caster, then record committed exhaust events.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::Any,
                filter: CastFilter::CasterOffTurn {
                    filter: &CastFilter::Any,
                },
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayers(PlayerSet::CastingPlayer, 4)],
        },
        TriggeredAbility {
            condition: TriggerCondition::ExhaustAbilityActivated,
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
