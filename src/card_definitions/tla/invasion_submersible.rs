// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: true,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ReturnToHand],
    }],
    activated: &[ActivatedAbility {
        exhaust: true,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::WaterbendCost(3),
            Instruction::AnimateSource(AnimationSpec {
                artifact: true,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: None,
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            }),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 3,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        exhaust_mask: 1,
        vehicle_stats: Some(Stats {
            power: 0,
            toughness: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
