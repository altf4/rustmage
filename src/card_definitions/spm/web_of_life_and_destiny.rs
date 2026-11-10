// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 5,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(1),
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(5),
            destination: LibraryPickDestination::Battlefield,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    extras: &CardExtras {
        has_convoke: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
