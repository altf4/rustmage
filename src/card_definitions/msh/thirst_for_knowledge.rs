// Draw first; optional artifact payment skips the ordinary two-card fallback.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 3),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Discard,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 16,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Skip(1),
            Instruction::Discard(PlayerSet::Controller, 2),
        ],
    ),
    ..CardRules::EMPTY
};
