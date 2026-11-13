// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[Instruction::Scry(2)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ScryOrSurveil,
        targets: TargetPlan::NONE,
        program: &[Instruction::LookLibraryTopMayCastFreeOncePerTurn],
    }],
    ..CardRules::EMPTY
};
