use crate::engine::Zone;

const COUNTERS: &[Instruction] = &[Instruction::PutCreatureCounters {
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    kind: CounterKind::PlusOne,
    amount: 1,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCombatRedScaled(AmountValue::SourcePower)],
        },
        TriggeredAbility {
            condition: TriggerCondition::CastFromExile(&TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Any,
                ordinal: None,
            }),
            targets: TargetPlan::NONE,
            program: COUNTERS,
        },
        TriggeredAbility {
            condition: TriggerCondition::ControlledPermanentEntersFrom(Zone::Exile),
            targets: TargetPlan::NONE,
            program: COUNTERS,
        },
    ],
    ..CardRules::EMPTY
};
