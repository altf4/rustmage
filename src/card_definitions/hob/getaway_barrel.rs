// Direct typed behavior from the full Oracle body and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 13,
                count: LibrarySelectionCount::RandomOne,
                filter: LibrarySelectionFilter::Kinds(1),
                optional: false,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(13),
            destination: LibraryPickDestination::Battlefield,
            public: true,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
