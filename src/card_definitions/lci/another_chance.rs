pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MayMill(2),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnUpToGraveyardToHand { kinds: 1 },
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
                amount: 2,
                optional_skip: None,
            },
        ],
    ),
    ..CardRules::EMPTY
};
