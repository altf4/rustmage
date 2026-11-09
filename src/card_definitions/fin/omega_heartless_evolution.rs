// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 8),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 1,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::Tap,
            Instruction::RememberCount(AmountValue::NonbasicLandsYouControl),
            Instruction::PutCountersScaled {
                subject: Subject::Targets,
                kind: CounterKind::Stun,
                amount: AmountValue::RememberedCount,
            },
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::RememberedCount),
        ],
    }],
    ..CardRules::EMPTY
};
