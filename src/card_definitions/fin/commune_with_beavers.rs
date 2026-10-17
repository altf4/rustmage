// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 3,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::Kinds(19),
            optional: true,
            reveal: true,
            remainder: LibraryRemainder::BottomAny,
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
    ..CardRules::EMPTY
};
