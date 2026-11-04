pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 9),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Stun,
                    amount: 3,
                },
                Instruction::Draw(PlayerSet::Controller, 3),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Subtype("Turtle"),
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::UntapSource],
        },
    ],
    ..CardRules::EMPTY
};
