// Optional resolution cost; payment and its replacements precede the draw.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeOrDiscard { nonland: false },
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 16,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Draw(PlayerSet::Controller, 2),
        ],
    ),
    ..CardRules::EMPTY
};
