// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDies {
            relation: CastRelation::You,
            other: true,
            nontoken: true,
            subtype: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Kinds(1),
                    optional: false,
                    reveal: true,
                    remainder: LibraryRemainder::BottomRandom,
                },
                look: LibraryLook::UntilKinds(1),
                destination: LibraryPickDestination::Hand,
                public: true,
                maximum_mana_value: None,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
