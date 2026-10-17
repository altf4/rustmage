// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 4,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::AnyOf(&[
                LibrarySelectionFilter::Subtype("Mutant"),
                LibrarySelectionFilter::Subtype("Ninja"),
                LibrarySelectionFilter::Subtype("Turtle"),
                LibrarySelectionFilter::Kinds(2),
            ]),
            optional: true,
            reveal: true,
            remainder: LibraryRemainder::BottomRandom,
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
