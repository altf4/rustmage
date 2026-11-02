// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::CreatureOrVehicle {
            relation: -1,
        })),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::LIFELINK.union(Keywords::INDESTRUCTIBLE),
        }],
    }],
    ..CardRules::EMPTY
};
