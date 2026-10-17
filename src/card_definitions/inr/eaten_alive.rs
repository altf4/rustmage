// Direct typed behavior from complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(object(1 | 64)), &[Instruction::Exile]),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
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
        }),
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 2, 0, 0, 0],
            generic: 3,
        }],
        mana_alternative_replaces_additional: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
