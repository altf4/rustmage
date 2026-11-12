// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::UntapDuringOtherUntapSteps],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 31,
            amount: 1,
        })],
    }],
    ..CardRules::EMPTY
};
