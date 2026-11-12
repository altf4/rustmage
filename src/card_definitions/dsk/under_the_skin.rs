// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ManifestDread,
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnGraveyardToHand {
                    kinds: 1 | 2 | 16 | 32 | 64 | 256,
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
                amount: 1,
                optional_skip: Some(0),
            },
        ],
    ),
    ..CardRules::EMPTY
};
