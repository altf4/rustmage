// Pinned XMage: formidable attack permission and sacrifice-paid artifact or
// enchantment removal are independent abilities.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Threshold {
                value: StepValue::TotalCreaturePower,
                minimum: Some(8),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlay {
            amount: 1,
            next_turn: false,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16 | 32,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::Destroy],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
