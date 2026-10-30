// Gated typed behavior; static loyalty grant requires full integration validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(5)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPlaneswalkerActivation(0)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: Some(-2),
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokens {
                token: Card(5298),
                amount: 1,
                tapped: false,
            },
            Instruction::Surveil(1),
        ],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
