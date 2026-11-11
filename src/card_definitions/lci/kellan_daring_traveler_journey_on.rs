pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 1,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(1),
                optional: false,
                reveal: true,
                remainder: LibraryRemainder::OptionalGraveyard,
            },
            look: LibraryLook::Fixed(1),
            destination: LibraryPickDestination::Hand,
            public: true,
            maximum_mana_value: Some(AmountValue::Fixed(3)),
        })],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Kellan, Daring Traveler // Journey On"),
        adventure_spell: Some(Card(5591)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
