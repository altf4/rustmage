const CONTROLLED: BattlefieldFilter = BattlefieldFilter {
    kinds: 0,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};
const VALLEY: &[BattlefieldFilter] = &[
    BattlefieldFilter {
        subtype: Some("Bird"),
        ..CONTROLLED
    },
    BattlefieldFilter {
        subtype: Some("Frog"),
        ..CONTROLLED
    },
    BattlefieldFilter {
        subtype: Some("Otter"),
        ..CONTROLLED
    },
    BattlefieldFilter {
        subtype: Some("Rat"),
        ..CONTROLLED
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrowPermanentGroup {
                filters: VALLEY,
                power: 1,
                toughness: 1,
            },
            Instruction::UntapPermanentGroup(VALLEY),
        ],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CastTiming(
                0,
                CastFilter::Noncreature,
            )],
        },
    ],
    extras: &CardExtras {
        cast_timing_grants: &[CastFilter::Noncreature],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
