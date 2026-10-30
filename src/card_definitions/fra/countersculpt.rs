// Direct Oracle and pinned XMage behavior; remains gated until rules validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[
            Instruction::Counter,
            Instruction::EnsureJaceToken,
            Instruction::EmpowerJace(1),
        ],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Behold,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: Some("Jace"),
            },
        }),
        additional_mana_alternative: Some(ManaCost {
            colored: [0; 6],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
