// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::EnsureJaceToken,
            Instruction::EmpowerJaceScaled(AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 1,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            }))),
        ],
    }],
    ..CardRules::EMPTY
};
