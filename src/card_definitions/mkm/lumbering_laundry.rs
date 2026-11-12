pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[Instruction::LookAtOpposingFaceDownCreaturesUntilEndOfTurn],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(generic(5))),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
