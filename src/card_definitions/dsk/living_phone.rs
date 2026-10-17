// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 5,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::CreaturePowerAtMost(2),
            optional: true,
            reveal: true,
            remainder: LibraryRemainder::BottomRandom,
        })],
    }],
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
