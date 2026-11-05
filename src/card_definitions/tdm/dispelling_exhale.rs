// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[
            Instruction::IfBeheld {
                beheld: true,
                skip: 2,
            },
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
        behold_cast: Some(BeholdCastSpec {
            subtype: "Dragon",
            decline_generic: 0,
            grants_flash: false,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
