use crate::engine::Zone;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
            kinds: 1,
            excluded_kinds: 2,
            relation: CastRelation::You,
            destinations: &[Zone::Graveyard],
            ..DepartureRule::ANY
        }),
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::AnimateEarthbendTargets,
            Instruction::PutCountersScaled {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: AmountValue::DeathEventPower,
            },
            Instruction::RegisterEarthbendReturn,
        ],
    }],
    ..CardRules::EMPTY
};
