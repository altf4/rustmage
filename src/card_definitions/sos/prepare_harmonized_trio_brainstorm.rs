// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 3),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::HandToTop,
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
    extras: &CardExtras {
        prepared_identity: Some("@prepare/harmonized_trio_brainstorm"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
