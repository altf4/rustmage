// A live granted ability protects current controlled Warrior tokens at own end step.
const GRANT: KeywordGrant = KeywordGrant {
    condition: Some(StatValue::ControllerEndStep),
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: Some("Warrior"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: Some(true),
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::CANT_BE_SACRIFICED,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::KeywordGrant(0, GRANT)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::Mobilize {
            token: Card(5255),
            amount: AmountValue::Fixed(2),
        }],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        keyword_grants: &[GRANT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
