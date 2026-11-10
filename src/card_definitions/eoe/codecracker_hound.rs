pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    extras: &CardExtras {
        warp: Some(&WarpSpec::fixed(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 2,
        })),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 2,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Any,
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::Graveyard,
            },
            look: LibraryLook::Fixed(2),
            destination: LibraryPickDestination::Hand,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
