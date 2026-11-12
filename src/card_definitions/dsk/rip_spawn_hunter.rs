// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::DistinctPower,
                filter: LibrarySelectionFilter::AnyOf(&[
                    LibrarySelectionFilter::Kinds(1),
                    LibrarySelectionFilter::Subtype("Vehicle"),
                ]),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Amount(AmountValue::SourcePower),
            destination: LibraryPickDestination::Hand,
            public: true,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
