// Current Mountain scaling and first qualifying declaration; full sources pinned.
const MOUNTAIN_BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some("Mountain"),
        basic: false,
        other: false,
        cards_only: false,
    }),
    stats: Stats {
        power: 2,
        toughness: 0,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 5),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, MOUNTAIN_BONUS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FirstAttackWithTotalPowerAtLeast(12),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::UntapCreatures(CreatureFilter {
                relation: -1,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: true,
                tokens: None,
                artifact: false,
            }),
            Instruction::AdditionalCombat,
        ],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[MOUNTAIN_BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
