pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::TapPermanents(2),
                filter: SacrificeFilter {
                    kinds: 1,
                    source_only: false,
                    other: false,
                    nonland: false,
                    tokens: None,
                    subtype: None,
                    keywords: Keywords::NONE,
                    artifact_enchantment_or_token: false,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 31,
            amount: 1,
        })],
    }],
    ..CardRules::EMPTY
};
