// Direct typed behavior from the pinned Oracle and XMage sources.
const CREATURES: BattlefieldFilter = BattlefieldFilter {
    kinds: 1,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutPermanentCountersScaled {
                filters: &[CREATURES],
                kind: CounterKind::PlusOne,
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Shrine"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OtherSubtypeEntersYou("Shrine"),
            targets: TargetPlan::NONE,
            program: &[Instruction::PutPermanentCountersScaled {
                filters: &[CREATURES],
                kind: CounterKind::PlusOne,
                amount: AmountValue::Fixed(1),
            }],
        },
    ],
    ..CardRules::EMPTY
};
