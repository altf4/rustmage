// Authored directly from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Counter target creature or legendary spell unless its controller pays {3}.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::CreatureOrLegendarySpell)),
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0, 0, 0, 0, 0, 0],
                    generic: 3,
                },
                subject: CounterSubject::Targets,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        SpellMode {
            text: "Target creature gets -5/-0 until end of turn.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: -5,
                toughness: 0,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
