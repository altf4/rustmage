// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LifeGained {
            first_only: false,
            own_turn: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutPermanentCountersScaled {
            filters: &[
                BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Pest"),
                    max_mana: u32::MAX,
                },
                BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Bat"),
                    max_mana: u32::MAX,
                },
                BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Insect"),
                    max_mana: u32::MAX,
                },
                BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Snake"),
                    max_mana: u32::MAX,
                },
                BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Spider"),
                    max_mana: u32::MAX,
                },
            ],
            kind: CounterKind::PlusOne,
            amount: AmountValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
