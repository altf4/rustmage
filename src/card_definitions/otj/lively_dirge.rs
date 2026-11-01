// Complete printed spree combinations, resolving in printed order.
const SEARCH: Instruction = Instruction::SearchRequired(SearchSpec {
    filter: SearchFilter::Any,
    maximum: 1,
    destination: SearchDestination::Graveyard,
    tapped: false,
    reveal: false,
});
const RETURN: Instruction = Instruction::ChooseGraveyardReturn {
    maximum: 2,
    total_mana_value: 4,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Search library into graveyard",
            targets: TargetPlan::NONE,
            program: &[SEARCH],
        },
        SpellMode {
            text: "Return creature cards",
            targets: TargetPlan::NONE,
            program: &[RETURN],
        },
        SpellMode {
            text: "Search library into graveyard + Return creature cards",
            targets: TargetPlan::NONE,
            program: &[SEARCH, RETURN],
        },
    ]),
    mode_costs: Some(&[generic(1), generic(2), generic(3)]),
    ..CardRules::EMPTY
};
