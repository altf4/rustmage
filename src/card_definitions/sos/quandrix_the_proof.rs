// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CascadeFromHand { ability: 1 }],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfCast,
            targets: TargetPlan::NONE,
            program: &[Instruction::Cascade],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Reflexive),
            targets: TargetPlan::NONE,
            program: &[Instruction::Cascade],
        },
    ],
    ..CardRules::EMPTY
};
