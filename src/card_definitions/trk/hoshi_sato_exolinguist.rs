// Pinned XMage: count distinct types of current non-Borg creatures, look that
// many cards, then select one privately and bottom the rest in random order.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Any,
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Amount(AmountValue::Stat(
                StatValue::DistinctControlledCreatureTypesExcept("Borg"),
            )),
            destination: LibraryPickDestination::Hand,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
