// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, cycling: false, origin: ActivationOrigin::Battlefield,
        cost: ActivationCost { colored: [0;6], generic: 0, alternatives: &[], additional: Some(AdditionalCost { kind: EffectChoiceKind::BlightCost(1), filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 1, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None } }), mana_component: true, tap: true, sacrifice: false, exile_self: false, discard: false, life: 0, remove: None, loyalty: None },
        targets: TargetPlan::NONE,
        program: &[Instruction::If { guard: StepGuard::Threshold { value: StepValue::Hand, minimum: Some(1), maximum: None }, skip: 2 },
            Instruction::Discard(PlayerSet::Controller, 1),
            Instruction::Draw(PlayerSet::Controller, 1),],
        sorcery: false, once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
