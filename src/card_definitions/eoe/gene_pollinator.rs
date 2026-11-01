// Typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::TapPermanents(1),
                filter: SacrificeFilter {
                    kinds: 0,
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
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 31,
            amount: 1,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
