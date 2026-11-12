pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::InstantSorcery,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Time,
                    amount: 1,
                },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::SourceCounterQuantity(CounterKind::Time),
                    minimum: 7,
                    skip: 2,
                },
                Instruction::RemoveAllSourceCounters(CounterKind::Time),
                Instruction::DamagePlayers(PlayerSet::Controller, 20),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCastSpell {
                prior_spells: false,
            }],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyNextInstantSorcery { ability: 1 }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
