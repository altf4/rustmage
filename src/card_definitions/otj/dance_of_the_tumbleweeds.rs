const SEARCH: Instruction = Instruction::Search(SearchSpec {
    filter: SearchFilter::AnyOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Desert")]),
    maximum: 1,
    destination: SearchDestination::Battlefield,
    tapped: false,
    reveal: false,
});
const ELEMENTAL: Instruction = Instruction::CreateSizedToken {
    token: Card::from_catalogue_id(5614),
    size: AmountValue::Stat(StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 2,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: false,
    })),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Search for a basic land or Desert",
            targets: TargetPlan::NONE,
            program: &[SEARCH],
        },
        SpellMode {
            text: "Create an Elemental",
            targets: TargetPlan::NONE,
            program: &[ELEMENTAL],
        },
        SpellMode {
            text: "Search for a land, then create an Elemental",
            targets: TargetPlan::NONE,
            program: &[SEARCH, ELEMENTAL],
        },
    ]),
    mode_costs: Some(&[generic(1), generic(3), generic(4)]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
