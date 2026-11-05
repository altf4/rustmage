// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
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
            min: 1,
            max: 2,
        }),
        &[
            Instruction::DistributeCounters {
                total: 2,
                kind: CounterKind::PlusOne,
            },
            Instruction::GrantCounterBearingCreatures(Keywords::VIGILANCE.union(Keywords::TRAMPLE)),
        ],
    ),
    extras: &CardExtras {
        harmonize_cost: Some(ManaCost {
            generic: 4,
            colored: [0, 0, 0, 0, 1, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
