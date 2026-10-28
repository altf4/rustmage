// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Creatures you control get +2/+0 until end of turn.",
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowPlayerCreatures {
                players: PlayerSet::Controller,
                power: 2,
                toughness: 0,
            }],
        },
        SpellMode {
            text: "Target creature gets +2/+0 and gains first strike until end of turn. Put a +1/+1 counter on it.",
            targets: one(TargetRule::Creature),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 2,
                    toughness: 0,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::FIRST_STRIKE,
                },
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            ],
        },
    ]),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
