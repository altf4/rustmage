pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(2))),
    aura: Some(AuraSpec {
        kinds: 2,
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
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: 1,
            other: false,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::ExileUntilSourceLeaves],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttachmentManaBonus(
            ManaProduction::any_color(1),
        )],
    }],
    ..CardRules::EMPTY
};
