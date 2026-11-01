// Cycling's X declaration is copied into its independent discard-trigger packet.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCycled,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::ManaValueExactly {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 48,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                amount: AmountValue::ChosenX,
                value: None,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::Destroy],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Hand,
        cycling: true,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 2, 0],
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ActivationX,
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
