// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Counter target spell unless its controller pays {4}.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0, 0, 0, 0, 0, 0],
                    generic: 4,
                },
                subject: CounterSubject::Targets,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        SpellMode {
            text: "Draw two cards, then discard a card.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::Draw(PlayerSet::Controller, 2),
                Instruction::Discard(PlayerSet::Controller, 1),
            ],
        },
    ]),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
