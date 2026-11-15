// Oracle and official release notes: sample distinct controlled land names once at resolution.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileOpponentTopChooseLandPlayWhileExiled(
            AmountValue::Stat(StatValue::DistinctNames(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 2,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
        )],
    }],
    ..CardRules::EMPTY
};
