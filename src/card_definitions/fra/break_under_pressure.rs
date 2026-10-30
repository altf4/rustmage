// Direct Oracle and pinned XMage behavior; gated pending rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeGreatestManaValue,
                players: PlayerSet::Targeted,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 65,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    ),
    ..CardRules::EMPTY
};
