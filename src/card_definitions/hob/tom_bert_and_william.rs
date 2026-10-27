// Directly authored from complete Oracle and pinned XMage, with CR611.2e entry typing.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter {
                    kinds: 1,
                    other: true,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::SacrificeCostPower),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCreatureDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnSourceAsArtifact],
    }],
    ..CardRules::EMPTY
};
