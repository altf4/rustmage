// Direct typed behavior from complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::SelfEnters,
            guard: StepGuard::Attacked,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 3,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Any,
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::Graveyard,
            },
            look: LibraryLook::Fixed(3),
            destination: LibraryPickDestination::Top,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
