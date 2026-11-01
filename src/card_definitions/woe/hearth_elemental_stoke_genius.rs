// Directly authored from Oracle and the pinned XMage implementation.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::GraveyardInstantSorceryAdventure,
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    }],
    extras: &CardExtras {
        adventure_identity: Some("Hearth Elemental // Stoke Genius"),
        adventure_spell: Some(Card(5485)),
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
