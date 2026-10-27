// Enroot is the prepared sorcery associated with Carnivorous Cultivator.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Land,
            maximum: 1,
            destination: SearchDestination::Graveyard,
            tapped: false,
            reveal: false,
        })],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/carnivorous_cultivator_enroot"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
