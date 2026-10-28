// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
const MODIFIER: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::GroupsOf {
        value: &StatValue::Count(CountFilter {
            zone: CountZone::Graveyard,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: true,
        }),
        size: std::num::NonZeroU16::new(7).unwrap(),
    },
    stats: Stats {
        power: 2,
        toughness: 0,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ScalingModifier(0, MODIFIER)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Mill(
            PlayerSet::Controller,
            MillAmount::Fixed(3),
        )],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        scaling_modifiers: &[MODIFIER],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
