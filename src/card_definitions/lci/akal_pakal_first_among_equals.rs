pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::Any,
            guard: StepGuard::ArtifactEnteredThisTurn,
        },
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
    ..CardRules::EMPTY
};
