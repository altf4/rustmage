// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1 | 16,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }))),
    aura: Some(AuraSpec {
        kinds: 1 | 16,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::TapAttached],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::AttachedUntap(UntapRestriction::ControllerStep),
            StaticAbilityFragment::AttachedActivationsProhibited,
        ],
    }],
    ..CardRules::EMPTY
};
