// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchDifferentNames(SearchSpec {
            filter: SearchFilter::Land,
            maximum: 4,
            destination: SearchDestination::BattlefieldAndOpponentChosenGraveyard { graveyard: 2 },
            tapped: true,
            reveal: true,
        })],
    ),
    ..CardRules::EMPTY
};
