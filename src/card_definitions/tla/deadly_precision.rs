// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Creature), &[Instruction::Destroy]),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter {
                artifact_enchantment_or_token: false,
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1 | 16,
                nonland: false,
                other: false,
                tokens: None,
                subtype: None,
            },
        }),
        additional_mana_alternative: Some(generic(4)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
