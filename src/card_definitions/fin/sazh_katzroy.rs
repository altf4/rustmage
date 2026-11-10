// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 1 },
                Instruction::Search(SearchSpec {
                    filter: SearchFilter::AnyOf(&[
                        SearchFilter::Subtype("Bird"),
                        SearchFilter::BasicLand,
                    ]),
                    maximum: 1,
                    destination: SearchDestination::Hand,
                    tapped: false,
                    reveal: true,
                }),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::DoubleCounterKind {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
