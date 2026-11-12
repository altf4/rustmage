pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 2,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::Any,
            optional: false,
            reveal: false,
            remainder: LibraryRemainder::Graveyard,
        })],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 1,
            },
            alternatives: &[
                ManaCost {
                    colored: [0, 1, 1, 0, 0, 0],
                    generic: 1,
                },
                ManaCost {
                    colored: [0, 0, 2, 0, 0, 0],
                    generic: 1,
                },
            ],
            modifier: DisguiseCostModifier::None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
