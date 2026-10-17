// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AnyOf(&[
                SearchFilter::BasicLand,
                SearchFilter::Subtype("Desert"),
            ]),
            maximum: 2,
            destination: SearchDestination::Battlefield,
            tapped: true,
            reveal: false,
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
