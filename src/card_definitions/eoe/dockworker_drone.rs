pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    enters: Counters {
        plus_one: 1,
        ..CardRules::EMPTY.enters
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::PutSourceCountersOnTargets],
    }],
    ..CardRules::EMPTY
};
