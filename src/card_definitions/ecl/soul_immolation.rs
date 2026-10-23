// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamagePlayersAndTheirCreatures {
            players: PlayerSet::Opponent,
            amount: AmountValue::ChosenX,
        }],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::BlightXCost,
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
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
