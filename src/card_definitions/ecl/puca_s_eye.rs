// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::ChooseColor,
            Instruction::SetColors {
                subject: Subject::Source,
                colors: ColorValue::Chosen,
                duration: AnimationDuration::Permanent,
            },
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::ControlColorsAtLeast(5),
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 3,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
