// Complete typed behavior from the pinned Oracle and XMage implementation.
const SPIRIT: Instruction = Instruction::CreateTokens {
    token: Card(5618),
    amount: 1,
    tapped: false,
};
const COUNTER: Instruction = Instruction::CounterUnlessPay {
    cost: generic(2),
    subject: CounterSubject::Targets,
    exile: false,
    unpaid_skip: 0,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Create a Spirit",
            targets: TargetPlan::NONE,
            program: &[SPIRIT],
        },
        SpellMode {
            text: "Counter unless its controller pays {2}",
            targets: one(TargetRule::Spell),
            program: &[COUNTER],
        },
        SpellMode {
            text: "Create a Spirit and counter unless its controller pays {2}",
            targets: one(TargetRule::Spell),
            program: &[SPIRIT, COUNTER],
        },
    ]),
    mode_costs: Some(&[generic(3), generic(1), generic(4)]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
