pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentKeywords {
            kinds: 1,
            other: false,
            keywords: Keywords::HASTE,
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 5,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(2),
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::Graveyard,
            },
            look: LibraryLook::Fixed(5),
            destination: LibraryPickDestination::BattlefieldTapped,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
