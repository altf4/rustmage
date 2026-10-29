// Direct Oracle and pinned XMage behavior; gated pending interaction validation.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, ReplacementSpec::NoncombatDamageToOpponentPlus(1))],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You, filter: CastFilter::Noncreature, ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow { subject: Subject::Source, power: 1, toughness: 1 }],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        replacement_effects: &[ReplacementSpec::NoncombatDamageToOpponentPlus(1)],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
