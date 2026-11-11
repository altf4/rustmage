// Complete typed behavior from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CommittedCrime,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: generic(1),
                skip: 1,
            },
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
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
            }),
        ],
    }],
    ..CardRules::EMPTY
};
