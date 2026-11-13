// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 2,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 2,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
