// Direct typed behavior from the paired Oracle and pinned XMage sources.
const POWER: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::ControlledLandCounters(CounterKind::PlusOne),
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[
            Instruction::AnimateEarthbendTargets,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
            Instruction::RegisterEarthbendReturn,
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, POWER)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(POWER),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
