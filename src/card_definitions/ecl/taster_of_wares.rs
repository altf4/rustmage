// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[Instruction::RevealHandForControlledCast(AmountValue::Stat(
            StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Goblin"),
                basic: false,
                other: false,
                cards_only: false,
            }),
        ))],
    }],
    ..CardRules::EMPTY
};
