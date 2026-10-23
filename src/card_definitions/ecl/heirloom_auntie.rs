// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    enters: Counters {
        minus_one: 2,
        plus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDies {
            relation: CastRelation::You,
            other: true,
            nontoken: false,
            subtype: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Surveil(1),
            Instruction::RemoveSourceCounterIfAny {
                kind: CounterKind::MinusOne,
                skip: 0,
            },
        ],
    }],
    ..CardRules::EMPTY
};
