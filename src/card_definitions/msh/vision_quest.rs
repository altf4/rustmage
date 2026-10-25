// Direct full Oracle program with one captured X for qualified search and entry.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchLibraryAndGraveyardWithXCounters {
            filter: SearchFilter::AllOf(&[SearchFilter::Artifact, SearchFilter::Creature]),
            counter: CounterKind::PlusOne,
            haste_at: Some(4),
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
