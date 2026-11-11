pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GiveGift,
            Instruction::DestroyAllCreaturesRememberOwned,
            Instruction::IfGift {
                promised: true,
                skip: 1,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnRememberedCreature,
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
                amount: 1,
                optional_skip: None,
            },
        ],
    ),
    extras: &CardExtras {
        gift: Some(GiftSpec {
            kind: GiftKind::Card,
            token: None,
            promised_targets: TargetPlan::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
