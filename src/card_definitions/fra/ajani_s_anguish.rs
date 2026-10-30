// Authored from pinned Oracle and complete XMage source; awaiting rules tests.
const CREATURE_TRAMPLE: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::TRAMPLE,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::KeywordGrant(0, CREATURE_TRAMPLE)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::DamageScaled(AmountValue::ChosenX)],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        x_symbols: 1,
        static_ability_groups: GROUPS,
        keyword_grants: &[CREATURE_TRAMPLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
