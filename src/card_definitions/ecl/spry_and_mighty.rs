// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::PowerDifferenceBoost,
            players: PlayerSet::Controller,
            filter: SacrificeFilter {
                artifact_enchantment_or_token: false,
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: false,
                tokens: None,
                subtype: None,
            },
            amount: 2,
            optional_skip: None,
        }],
    ),
    ..CardRules::EMPTY
};
