const fn optional(kinds: u8) -> TargetSpec {
    TargetSpec {
        rule: object(kinds),
        min: 0,
        max: 1,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[optional(16), optional(1), optional(32), optional(64)]),
        &[Instruction::DestroyAllTargets],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::CollectEvidenceForTargetManaValue,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
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
