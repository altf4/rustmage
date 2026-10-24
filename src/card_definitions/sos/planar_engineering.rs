// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 2,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
                amount: 2,
                optional_skip: None,
            },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 4,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: true,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
