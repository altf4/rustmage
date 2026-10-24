// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::AnyOf(&[
                    LibrarySelectionFilter::Kinds(2),
                    LibrarySelectionFilter::HasX,
                ]),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(5),
            destination: LibraryPickDestination::Hand,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 2, 0, 0, 1, 0],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
