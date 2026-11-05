// Directly authored from the complete pinned Oracle and XMage implementation.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::ControlledUntappedSubtypeKeywords {
        subtype: "Dragon",
        keywords: Keywords::HEXPROOF,
    }],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
            look: 6,
            count: LibrarySelectionCount::Fixed(1),
            filter: LibrarySelectionFilter::AnyOf(&[
                LibrarySelectionFilter::Kinds(4 | 8),
                LibrarySelectionFilter::Subtype("Dragon"),
            ]),
            optional: true,
            reveal: true,
            remainder: LibraryRemainder::BottomRandom,
        })],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
