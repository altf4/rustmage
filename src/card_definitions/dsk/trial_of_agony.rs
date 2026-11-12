pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 2,
            max: 2,
        }),
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::DamageAmongTargetGroup {
                    group: 0,
                    amount: 5,
                },
                players: PlayerSet::Opponent,
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
            Instruction::GrantKeywordsToUnchosenTargets {
                group: 0,
                keywords: Keywords::CANT_BLOCK,
            },
        ],
    ),
    ..CardRules::EMPTY
};
