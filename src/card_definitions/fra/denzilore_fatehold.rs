// Directly authored from pinned Oracle and XMage; pending focused validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ScryOrSurveil,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutPermanentCountersScaled {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            kind: CounterKind::PlusOne,
            amount: AmountValue::Fixed(1),
        }],
    }],
    keywords: Keywords::FLYING.union(Keywords::FLASH),
    ..CardRules::EMPTY
};
