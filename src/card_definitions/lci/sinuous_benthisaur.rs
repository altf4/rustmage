const CAVES: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Cave"),
    basic: false,
    other: false,
    cards_only: false,
};
const LOOK: StatValue = StatValue::Sum(&[
    StatValue::Count(CAVES),
    StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        cards_only: true,
        ..CAVES
    }),
]);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 0,
                count: LibrarySelectionCount::Fixed(2),
                filter: LibrarySelectionFilter::Any,
                optional: false,
                reveal: false,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Amount(AmountValue::Stat(LOOK)),
            destination: LibraryPickDestination::Hand,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    ..CardRules::EMPTY
};
