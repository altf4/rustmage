pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 6),
    keywords: Keywords::TRAMPLE
        .union(Keywords::VIGILANCE)
        .union(Keywords::HASTE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::All,
                filter: LibrarySelectionFilter::AllOf(&[
                    LibrarySelectionFilter::Kinds(1),
                    LibrarySelectionFilter::Subtype("Dinosaur"),
                ]),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Amount(AmountValue::DamageEventAmount),
            destination: LibraryPickDestination::Battlefield,
            public: true,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
