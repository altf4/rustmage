// Pinned Oracle and complete XMage class: each mode has its own target plan.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Interfere",
            targets: one(TargetRule::Spell),
            program: &[Instruction::CounterUnlessPay {
                cost: generic(3),
                subject: CounterSubject::Targets,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        SpellMode {
            text: "Observe",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[
                Instruction::Tap,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::Stun,
                    amount: 1,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
