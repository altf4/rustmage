// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 4,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::AnyOf(&[
                LibrarySelectionFilter::Subtype("Goblin"),
                LibrarySelectionFilter::Subtype("Swamp"),
                LibrarySelectionFilter::Subtype("Mountain"),
            ]),
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
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 1, 2, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 2, 1, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 3, 0, 0, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
