pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 3),
            Instruction::LoseLife(PlayerSet::Opponent, 3),
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::FilteredCost {
                action: CostAction::Sacrifice,
                any: &[CostMatch::DealtDamageThisTurn],
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
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
