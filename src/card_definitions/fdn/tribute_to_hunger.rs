// Directly authored from complete Oracle and pinned XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Targeted,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::SacrificedToughness),
        ],
    ),
    ..CardRules::EMPTY
};
