// Partial target-choice draft only; Oracle source lacks a pinned XMage class.
// Exile and free-copy casting are drafted; keep gated pending integration validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Grouped {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                constraint: TargetGroupConstraint::DistinctCardTypeSlots,
            },
            min: 0,
            max: 16,
        }),
        program: &[Instruction::ExileTargetsCopyFreeWithBudget { mana_value: 6 }],
    }],
    keywords: Keywords::FLYING,
    ..CardRules::EMPTY
};
