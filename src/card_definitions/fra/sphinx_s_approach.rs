// Complete Oracle and pinned XMage rules; admission is conditional on candidate validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileSourceAndNamed {
                    name: "Sphinx's Approach",
                    amount: 4,
                },
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 4,
                optional_skip: Some(1),
            },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::AllOf(&[
                    SearchFilter::Creature,
                    SearchFilter::Subtype("Sphinx"),
                ]),
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: false,
                reveal: false,
            }),
        ],
    ),
    deck_limit: None,
    ..CardRules::EMPTY
};
