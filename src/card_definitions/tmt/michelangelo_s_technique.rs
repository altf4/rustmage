// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 8,
                count: LibrarySelectionCount::ManaValueBudget {
                    maximum_cards: 2,
                    maximum_total: 6,
                },
                filter: LibrarySelectionFilter::Kinds(1),
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(8),
            destination: LibraryPickDestination::Battlefield,
            public: false,
            maximum_mana_value: None,
        })],
    ),
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 3,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
