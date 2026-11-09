// Directly authored from the paired Oracle and pinned XMage sources.
const UP_TO_ONE_NONLAND: TargetPlan = TargetPlan::Single(TargetSpec {
    rule: TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 0,
        relation: -1,
        other: false,
        nonland: true,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    min: 0,
    max: 1,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(9, 9),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: UP_TO_ONE_NONLAND,
            program: &[Instruction::Destroy],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: UP_TO_ONE_NONLAND,
            program: &[Instruction::Destroy],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayersScaled(
                PlayerSet::Opponent,
                AmountValue::Stat(StatValue::TotalManaValue(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: true,
                    cards_only: false,
                })),
            )],
        },
    ],
    ..CardRules::EMPTY
};
