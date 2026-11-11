const SCALE: ScalingModifier = ScalingModifier {
    subject: Subject::Attached,
    value: StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 1 | 2 | 16 | 32 | 64 | 256,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    }),
    stats: Stats {
        power: -1,
        toughness: 0,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 2,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ScalingModifier(0, SCALE)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::CreatureOrVehicle { relation: -1 })),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayMill(2),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
        ],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        aura_kind_or_subtype: Some("Vehicle"),
        scaling_modifiers: &[SCALE],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
