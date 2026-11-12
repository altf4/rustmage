pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileControllerLibraryTopLinkedFaceDown(1)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            colored: [0, 0, 0, 1, 0, 0],
            mana_component: true,
            sacrifice: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::DiscardHand,
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
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnLinkedExileToHand],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
