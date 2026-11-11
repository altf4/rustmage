pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutSourceCounterReflexive {
            kind: CounterKind::Coin,
            amount: 1,
            ability: 0,
        }],
        origin: ActivationOrigin::Battlefield,
        sorcery: true,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::ManaValueExactly {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 4 | 8,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            amount: AmountValue::SourceCounterQuantity(CounterKind::Coin),
            value: None,
        })),
        program: &[Instruction::CastTargetedGraveyardFree],
    }],
    ..CardRules::EMPTY
};
