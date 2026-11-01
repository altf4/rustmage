pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::ExileCreatureAndGraveyard,
            players: PlayerSet::Targeted,
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
            amount: 1,
            optional_skip: None,
        }],
    ),
    ..CardRules::EMPTY
};
