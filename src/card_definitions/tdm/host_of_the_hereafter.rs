// Source-audited direct typed definition; see the paired provenance record.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        fixed_entry_counters: &[(CounterKind::PlusOne, 2)],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesWithCounters {
            relation: CastRelation::You,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::PutEventCountersOnTargets],
    }],
    ..CardRules::EMPTY
};
