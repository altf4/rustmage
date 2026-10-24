// Directly authored from the full pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Spell),
        &[
            Instruction::IfTeamwork { skip: 2 },
            Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 4,
                },
                subject: CounterSubject::Targets,
                exile: false,
                unpaid_skip: 0,
            },
            Instruction::Skip(1),
            Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                subject: CounterSubject::Targets,
                exile: false,
                unpaid_skip: 0,
            },
        ],
    ),
    extras: &CardExtras {
        teamwork: Some(TeamworkSpec {
            minimum: 2,
            paid_targets: None,
            normal_modes: 0,
            paid_modes: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
