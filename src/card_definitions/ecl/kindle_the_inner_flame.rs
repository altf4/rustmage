// Direct typed behavior from the full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[Instruction::CreateCopyTokenWithExceptions {
            subject: Subject::Targets,
            amount: 1,
            exceptions: CopyExceptions::HASTE_AND_SACRIFICE,
        }],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
        }),
        flashback_additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::BeholdMany(3),
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: Some("Elemental"),
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
