const CAST_SACRIFICE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1 | 32,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Opponent,
                filter: SacrificeFilter {
                    kinds: 32,
                    tokens: Some(false),
                    ..CAST_SACRIFICE
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Opponent,
                filter: SacrificeFilter {
                    kinds: 1,
                    tokens: Some(false),
                    ..CAST_SACRIFICE
                },
                amount: 1,
                optional_skip: None,
            },
        ],
    }],
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: CAST_SACRIFICE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
