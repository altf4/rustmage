// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksAlone,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::GrowCreatureActionScaled(AmountValue::Stat(
            StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Kithkin"),
                basic: false,
                other: false,
                cards_only: false,
            }),
        ))],
    }],
    ..CardRules::EMPTY
};
