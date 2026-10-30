// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    enters: Counters {
        plus_one: 1,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::creature_you_control(),
            min: 0,
            max: 1,
        }),
        program: &[Instruction::PutSourceCountersOnTargets],
    }],
    ..CardRules::EMPTY
};
