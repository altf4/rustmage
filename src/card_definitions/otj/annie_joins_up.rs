const OPPOSING_CREATURE_OR_PLANESWALKER: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1 | 64,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(OPPOSING_CREATURE_OR_PLANESWALKER),
        program: &[Instruction::Damage(5)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledLegendaryCreature,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
