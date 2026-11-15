// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 4,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::ExcludingKinds(1 | 2),
            optional: true,
            reveal: true,
            remainder: LibraryRemainder::BottomAny,
        })],
    }],
    ..CardRules::EMPTY
};
