// Full frozen Oracle and direct pinned Java; the self-granted ability is layer7b.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::NoMaximumHandSize],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    keywords: Keywords::REACH.union(Keywords::VIGILANCE),
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
                target: CastTarget::Creature(CastRelation::You),
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::GrantSelfAbility(
                crate::engine::SelfGrantedAbilityKind::ControllerHandPower,
            ),
        ],
    }],
    ..CardRules::EMPTY
};
