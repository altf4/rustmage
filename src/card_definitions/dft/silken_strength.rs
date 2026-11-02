// Creature-or-Vehicle legality and attachment bonuses share the frozen layered query.
const AURA: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 1,
            toughness: 2,
        },
        keywords: Keywords::REACH,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 3,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::AttachmentStats(Stats {
            power: 1,
            toughness: 2,
        }),
        StaticAbilityFragment::AttachmentKeywords(Keywords::REACH),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::CreatureOrVehicle { relation: -1 })),
    aura: Some(AURA),
    keywords: Keywords::FLASH,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapAttached],
    }],
    extras: &CardExtras {
        aura: Some(AURA),
        aura_kind_or_subtype: Some("Vehicle"),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
