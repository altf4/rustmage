// Direct typed behavior from full Oracle and complete pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDamaged,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: AmountValue::DamageEventAmount,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Creatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    qualification: DamageQualification::Counter,
                },
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 1, 0, 0, 0, 0],
                        generic: 0,
                    },
                    skip: 1,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
