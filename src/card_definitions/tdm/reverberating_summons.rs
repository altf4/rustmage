// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::Any,
            guard: StepGuard::Threshold {
                value: StepValue::SpellsCast,
                minimum: Some(2),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AnimateSource(AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(&["Monk"]),
            base: Some((3, 3)),
            keywords: Keywords::HASTE,
            duration: AnimationDuration::EndOfTurn,
        })],
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
        program: &[Instruction::Draw(PlayerSet::Controller, 2)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
