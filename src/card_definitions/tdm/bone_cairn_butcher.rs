const GRANT: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: true,
        tokens: Some(true),
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::DEATHTOUCH,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::KeywordGrant(0, GRANT)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::Mobilize {
            token: Card(5255),
            amount: AmountValue::Fixed(2),
        }],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        keyword_grants: &[GRANT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
