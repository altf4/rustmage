// One static ability starts in layer four and continues through layers six/seven.
const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 2,
    },
    keywords: Keywords::FIRST_STRIKE.union(Keywords::VIGILANCE),
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Type,
    fragments: &[
        StaticAbilityFragment::AttachedSubtype("Soldier"),
        StaticAbilityFragment::AttachedLegendary,
        StaticAbilityFragment::AttachmentStats(BONUS.stats),
        StaticAbilityFragment::AttachmentKeywords(BONUS.keywords),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: BONUS,
    }),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedAttacksOrBlocks,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 16,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Equipment"),
            },
            min: 0,
            max: usize::MAX,
        }),
        program: &[Instruction::AttachEquipmentToAttached { group: 0 }],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        attached_subtype: Some("Soldier"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
