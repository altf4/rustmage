const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Type,
    fragments: &[StaticAbilityFragment::AttachedCharacteristics(
        &AttachedCharacteristics {
            name: None,
            form: PermanentForm::AddCreature,
            creature_types: &["Golem"],
            colors: &[],
            base: Some((5, 4)),
        },
    )],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(16))),
    aura: Some(AuraSpec {
        kinds: 16,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::Discover(3)],
    }],
    ..CardRules::EMPTY
};
