// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CollectGraveyard,
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(4 | 8),
                count: MilledCount::All,
                destination: MilledDestination::Hand,
            }),
            Instruction::NoMaximumHandSize,
            Instruction::ExileResolvingSpell,
        ],
    ),
    ..CardRules::EMPTY
};
