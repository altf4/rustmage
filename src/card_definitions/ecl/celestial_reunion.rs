// Direct typed behavior from complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchCreatureManaValue {
            maximum: AmountValue::ChosenX,
            destination: SearchDestination::BeheldCreatureType,
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::OptionalBeholdType(2),
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
