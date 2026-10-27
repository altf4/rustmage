const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Sum(&[
        StatValue::AtLeast {
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
            minimum: 7,
        },
        StatValue::AtLeast {
            value: &StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 1,
                kind: 0,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: true,
            }),
            minimum: 7,
        },
    ]),
    stats: Stats {
        power: 2,
        toughness: 0,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::VIGILANCE,
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, BONUS)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::You,
            ordinal: Some(2),
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        program: &[Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(3))],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
