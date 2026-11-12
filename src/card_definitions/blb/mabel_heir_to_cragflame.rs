pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupBonus {
            filters: &[CreatureFilter {
                relation: 0,
                other: true,
                subtype: Some("Mouse"),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            }],
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5619),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
