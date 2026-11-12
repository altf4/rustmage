// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::DEFENDER,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::DEFENDER)],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(1)),
            Instruction::RecoverMilled(RecoveryFilter::Land),
            Instruction::IfAmountAtLeast {
                value: AmountValue::MilledSubtypeCount("Lesson"),
                minimum: 1,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
