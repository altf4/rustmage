// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Spell), &[Instruction::Counter]),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::BlightCost(2),
            filter: SacrificeFilter {
                kinds: 1,
                keywords: Keywords::NONE,
                source_only: false,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        cost_alternatives: &[ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 1,
        }],
        mana_alternative_replaces_additional: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
