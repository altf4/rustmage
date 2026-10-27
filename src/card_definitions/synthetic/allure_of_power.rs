pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 2)],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(2893)),
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
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
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
