const SEARCH: Instruction = Instruction::Search(SearchSpec {
    filter: SearchFilter::Kinds(16),
    maximum: 1,
    destination: SearchDestination::Hand,
    tapped: false,
    reveal: true,
});
const RETURN: TargetPlan = one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 16,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Search your library for an artifact card.",
            targets: TargetPlan::NONE,
            program: &[SEARCH],
        },
        SpellMode {
            text: "Return target artifact card from your graveyard to your hand.",
            targets: RETURN,
            program: &[Instruction::ReturnToHand],
        },
        SpellMode {
            text: "Choose both",
            targets: RETURN,
            program: &[SEARCH, Instruction::ReturnToHand],
        },
    ]),
    ..CardRules::EMPTY
};
