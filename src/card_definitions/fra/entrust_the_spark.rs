// Authored from pinned Oracle and complete XMage source; awaiting rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 64,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::Kinds(64),
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: false,
                reveal: false,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
