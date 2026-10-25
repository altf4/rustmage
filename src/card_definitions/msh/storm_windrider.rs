// Full frozen Oracle and pinned direct Java; cast subjects are not trigger targets.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::FlyingCombatRestriction],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Targeted {
                filter: &CastFilter::Any,
                target: CastTarget::Creature(CastRelation::Any),
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantCastCreatureKeywords(Keywords::FLYING)],
    }],
    ..CardRules::EMPTY
};
