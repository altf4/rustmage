// Direct typed behavior from the complete Oracle body and pinned XMage sources.
const BLINK_TARGET: TargetPlan = TargetPlan::Single(TargetSpec {
    rule: TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 3,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    min: 0,
    max: 1,
});
const BLINK: &[Instruction] = &[Instruction::BlinkUntilEndStep { under_owner: true }];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: BLINK_TARGET,
            program: BLINK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: BLINK_TARGET,
            program: BLINK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: BLINK_TARGET,
            program: BLINK,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: BLINK_TARGET,
            program: BLINK,
        },
    ],
    ..CardRules::EMPTY
};
