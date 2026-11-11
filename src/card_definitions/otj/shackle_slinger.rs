const OPPOSING_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: Some(2),
        },
        targets: one(OPPOSING_CREATURE),
        program: &[
            Instruction::RememberTargetTapped,
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 2,
            },
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Stun,
                amount: 1,
            },
            Instruction::Skip(1),
            Instruction::Tap,
        ],
    }],
    ..CardRules::EMPTY
};
