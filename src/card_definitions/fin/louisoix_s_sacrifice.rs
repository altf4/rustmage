pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Union(&[
            TargetRule::Ability,
            TargetRule::SpellKinds {
                kinds: 0,
                excluded: 1,
            },
        ])),
        &[Instruction::Counter],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::FilteredCost {
                action: CostAction::Sacrifice,
                any: &[CostMatch::Legendary],
            },
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
        additional_mana_alternative: Some(ManaCost {
            generic: 2,
            ..ManaCost::ZERO
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
