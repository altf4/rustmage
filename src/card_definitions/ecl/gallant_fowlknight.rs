// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
const YOUR_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrowCreatures {
                filter: YOUR_CREATURES,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantCreatures {
                filter: CreatureFilter {
                    subtype: Some("Kithkin"),
                    ..YOUR_CREATURES
                },
                keywords: Keywords::FIRST_STRIKE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
