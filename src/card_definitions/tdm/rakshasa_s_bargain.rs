// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 4,
            count: LibrarySelectionCount::Fixed(2),
            filter: LibrarySelectionFilter::Any,
            optional: false,
            reveal: false,
            remainder: LibraryRemainder::Graveyard,
        })],
    ),
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
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 1, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 0, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 1, 1, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 1, 1, 0, 1, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
