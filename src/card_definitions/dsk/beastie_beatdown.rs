// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
                    minimum: Some(4),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::PutCountersOnTargetGroup {
                group: 0,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
            Instruction::GroupPowerDamage {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
