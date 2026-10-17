// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Look at the top four cards of your library. Put one of them into your hand and the rest into your graveyard.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Any,
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::Graveyard,
            })],
        },
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
