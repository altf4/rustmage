// Oracle and official Star Trek release notes reviewed; XMage class absent at the pinned revision.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryToSacrificeAtNextOwnEndStep(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::All,
                filter: LibrarySelectionFilter::Kinds(2),
                optional: false,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::UntilKindsCount { kinds: 2, count: 2 },
            destination: LibraryPickDestination::Battlefield,
            public: true,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
