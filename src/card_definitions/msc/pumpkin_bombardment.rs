pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Creature), &[Instruction::Damage(3)]),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 0,
        }],
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Discard,
            filter: SacrificeFilter {
                kinds: 0,
                source_only: false,
                other: false,
                nonland: false,
                tokens: None,
                subtype: None,
                keywords: Keywords::NONE,
                artifact_enchantment_or_token: false,
            },
        }),
        additional_mana_alternative: Some(ManaCost {
            generic: 2,
            ..ManaCost::ZERO
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
